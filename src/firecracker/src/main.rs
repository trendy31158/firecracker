// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

extern crate backtrace;
#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate api_server;
extern crate libc;
extern crate utils;
#[macro_use]
extern crate logger;
extern crate mmds;
extern crate seccomp;
extern crate vmm;

use backtrace::Backtrace;
use clap::{App, Arg};

use std::fs;
use std::io;
use std::panic;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use std::sync::{Arc, RwLock};
use std::thread;

use api_server::{ApiServer, Error, VmmRequest, VmmResponse};
use logger::{Metric, LOGGER, METRICS};
use mmds::MMDS;
use utils::eventfd::EventFd;
use utils::terminal::Terminal;
use utils::validators::validate_instance_id;
use vmm::controller::VmmController;
use vmm::resources::VmResources;
use vmm::signal_handler::register_signal_handlers;
use vmm::vmm_config;
use vmm::vmm_config::instance_info::InstanceInfo;
use vmm::EventLoopExitReason;

const DEFAULT_API_SOCK_PATH: &str = "/tmp/firecracker.socket";
const DEFAULT_INSTANCE_ID: &str = "anonymous-instance";

fn main() {
    LOGGER
        .preinit(Some(DEFAULT_INSTANCE_ID.to_string()))
        .expect("Failed to register logger");

    if let Err(e) = register_signal_handlers() {
        error!("Failed to register signal handlers: {}", e);
        process::exit(i32::from(vmm::FC_EXIT_CODE_GENERIC_ERROR));
    }

    // We need this so that we can reset terminal to canonical mode if panic occurs.
    let stdin = io::stdin();

    // Start firecracker by setting up a panic hook, which will be called before
    // terminating as we're building with panic = "abort".
    // It's worth noting that the abort is caused by sending a SIG_ABORT signal to the process.
    panic::set_hook(Box::new(move |info| {
        // We're currently using the closure parameter, which is a &PanicInfo, for printing the
        // origin of the panic, including the payload passed to panic! and the source code location
        // from which the panic originated.
        error!("Firecracker {}", info);
        if let Err(e) = stdin.lock().set_canon_mode() {
            error!(
                "Failure while trying to reset stdin to canonical mode: {}",
                e
            );
        }

        METRICS.vmm.panic_count.inc();
        let bt = Backtrace::new();
        error!("{:?}", bt);

        // Log the metrics before aborting.
        if let Err(e) = LOGGER.log_metrics() {
            error!("Failed to log metrics while panicking: {}", e);
        }
    }));

    let cmd_arguments = App::new("firecracker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Launch a microvm.")
        .arg(
            Arg::with_name("api_sock")
                .long("api-sock")
                .help("Path to unix domain socket used by the API")
                .takes_value(true)
                .default_value(DEFAULT_API_SOCK_PATH),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .help("MicroVM unique identifier")
                .takes_value(true)
                .default_value(DEFAULT_INSTANCE_ID)
                .validator(|s: String| -> Result<(), String> {
                    validate_instance_id(&s).map_err(|e| format!("{}", e))
                }),
        )
        .arg(
            Arg::with_name("seccomp-level")
                .long("seccomp-level")
                .help(
                    "Level of seccomp filtering.\n
                            - Level 0: No filtering.\n
                            - Level 1: Seccomp filtering by syscall number.\n
                            - Level 2: Seccomp filtering by syscall number and argument values.\n
                        ",
                )
                .takes_value(true)
                .default_value("2")
                .possible_values(&["0", "1", "2"]),
        )
        .arg(
            Arg::with_name("start-time-us")
                .long("start-time-us")
                .takes_value(true)
                .hidden(true),
        )
        .arg(
            Arg::with_name("start-time-cpu-us")
                .long("start-time-cpu-us")
                .takes_value(true)
                .hidden(true),
        )
        .arg(
            Arg::with_name("config-file")
                .long("config-file")
                .help("Path to a file that contains the microVM configuration in JSON format.")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("no-api")
                .long("no-api")
                .help("Optional parameter which allows starting and using a microVM without an active API socket.")
                .takes_value(false)
                .required(false)
                .requires("config-file")
        )
        .get_matches();

    let bind_path = cmd_arguments
        .value_of("api_sock")
        .map(PathBuf::from)
        .expect("Missing argument: api_sock");

    // It's safe to unwrap here because clap's been provided with a default value
    let instance_id = cmd_arguments.value_of("id").unwrap().to_string();

    // It's safe to unwrap here because clap's been provided with a default value,
    // and allowed values are guaranteed to parse to u32.
    let seccomp_level = cmd_arguments
        .value_of("seccomp-level")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let start_time_us = cmd_arguments.value_of("start-time-us").map(|s| {
        s.parse::<u64>()
            .expect("'start-time-us' parameter expected to be of 'u64' type.")
    });

    let start_time_cpu_us = cmd_arguments.value_of("start-time-cpu-us").map(|s| {
        s.parse::<u64>()
            .expect("'start-time-cpu_us' parameter expected to be of 'u64' type.")
    });

    let vmm_config_json = cmd_arguments
        .value_of("config-file")
        .map(fs::read_to_string)
        .map(|x| x.expect("Unable to open or read from the configuration file"));

    let no_api = cmd_arguments.is_present("no-api");

    LOGGER.set_instance_id(instance_id.clone());
    let api_shared_info = Arc::new(RwLock::new(InstanceInfo {
        id: instance_id,
        started: false,
        vmm_version: crate_version!().to_string(),
    }));

    let request_event_fd = EventFd::new(libc::EFD_NONBLOCK)
        .map_err(Error::Eventfd)
        .expect("Cannot create API Eventfd.");
    let (to_vmm, from_api) = channel();
    let (to_api, from_vmm) = channel();

    // Api enabled.
    if !no_api {
        // MMDS only supported with API.
        let mmds_info = MMDS.clone();
        let vmm_shared_info = api_shared_info.clone();
        let to_vmm_event_fd = request_event_fd.try_clone().unwrap();

        thread::Builder::new()
            .name("fc_api".to_owned())
            .spawn(move || {
                match ApiServer::new(
                    mmds_info,
                    vmm_shared_info,
                    to_vmm,
                    from_vmm,
                    to_vmm_event_fd,
                )
                .expect("Cannot create API server")
                .bind_and_run(
                    bind_path,
                    start_time_us,
                    start_time_cpu_us,
                    seccomp_level,
                ) {
                    Ok(_) => (),
                    Err(Error::Io(inner)) => match inner.kind() {
                        io::ErrorKind::AddrInUse => {
                            panic!("Failed to open the API socket: {:?}", Error::Io(inner))
                        }
                        _ => panic!(
                            "Failed to communicate with the API socket: {:?}",
                            Error::Io(inner)
                        ),
                    },
                    Err(eventfd_err @ Error::Eventfd(_)) => {
                        panic!("Failed to open the API socket: {:?}", eventfd_err)
                    }
                }
            })
            .expect("API thread spawn failed.");
    }

    run(
        api_shared_info,
        request_event_fd,
        from_api,
        to_api,
        seccomp_level,
        vmm_config_json,
    );
}

/// Creates, starts then controls a vmm.
///
/// # Arguments
///
/// * `api_shared_info` - A parameter for storing information on the VMM (e.g the current state).
/// * `api_event_fd` - An event fd used for receiving API associated events.
/// * `from_api` - The receiver end point of the communication channel.
/// * `seccomp_level` - The level of seccomp filtering used. Filters are loaded before executing
///                     guest code. Can be one of 0 (seccomp disabled), 1 (filter by syscall
///                     number) or 2 (filter by syscall number and argument values).
/// * `config_json` - Optional parameter that can be used to configure the guest machine without
///                   using the API socket.
fn run(
    api_shared_info: Arc<RwLock<InstanceInfo>>,
    api_event_fd: EventFd,
    from_api: Receiver<VmmRequest>,
    to_api: Sender<VmmResponse>,
    seccomp_level: u32,
    config_json: Option<String>,
) {
    // The driving epoll engine.
    let mut epoll_context = vmm::EpollContext::new().expect("Cannot create the epoll context.");
    epoll_context
        .add_epollin_event(&api_event_fd, vmm::EpollDispatch::VmmActionRequest)
        .expect("Cannot add vmm control_fd to epoll.");

    // These will be populated from JSON or API.
    let mut vm_resources = VmResources::default();

    // Build and start the microVM.
    let vmm = configure_and_build_microvm(
        &api_event_fd,
        &from_api,
        &to_api,
        seccomp_level,
        config_json,
        &mut epoll_context,
        &mut vm_resources,
    );

    api_shared_info.write().unwrap().started = true;

    let mut vm_controller: VmmController = VmmController::new(epoll_context, vm_resources, vmm);

    let exit_code = loop {
        match vm_controller.run_event_loop() {
            Err(e) => {
                error!("Abruptly exited VMM control loop: {:?}", e);
                break vmm::FC_EXIT_CODE_GENERIC_ERROR;
            }
            Ok(exit_reason) => match exit_reason {
                EventLoopExitReason::Break => {
                    info!("Gracefully terminated VMM control loop");
                    break vmm::FC_EXIT_CODE_OK;
                }
                EventLoopExitReason::ControlAction => {
                    if let Err(exit_code) =
                        vmm_control_event(&mut vm_controller, &api_event_fd, &from_api, &to_api)
                    {
                        break exit_code;
                    }
                }
            },
        };
    };

    vm_controller.stop(i32::from(exit_code));
}

/// Builds and starts a microVM. It either uses a config json or receives API requests to
/// get the microVM configuration.
///
/// Returns a running `Vmm` object.
fn configure_and_build_microvm(
    api_event_fd: &EventFd,
    from_api: &Receiver<VmmRequest>,
    to_api: &Sender<VmmResponse>,
    seccomp_level: u32,
    config_json: Option<String>,
    // FIXME: epoll context can be polluted by failing boot attempts
    epoll_context: &mut vmm::EpollContext,
    vm_resources: &mut VmResources,
) -> vmm::Vmm {
    use vmm::{ErrorKind, VmmActionError};

    // Start a microVm configured from command-line JSON.
    if let Some(ref json) = config_json {
        *vm_resources =
            VmResources::from_json(json, crate_version!().to_string()).unwrap_or_else(|err| {
                error!(
                    "Setting configuration for VMM from one single json failed: {:?}",
                    err
                );
                process::exit(i32::from(vmm::FC_EXIT_CODE_BAD_CONFIGURATION));
            });
        match vmm::builder::build_microvm(&vm_resources, epoll_context, seccomp_level) {
            Ok(vmm) => {
                info!("Successfully started microvm that was configured from one single json");
                return vmm;
            }
            Err(err) => {
                error!(
                    "Building VMM configured from cmdline json failed: {:?}",
                    err
                );
                process::exit(i32::from(vmm::FC_EXIT_CODE_BAD_CONFIGURATION));
            }
        };
    }

    // Configure and start microVM through successive API calls.
    // Iterate through API calls to configure microVm.
    // The loop breaks when a microVM is successfully started, and returns a running Vmm.
    loop {
        let mut built_vmm = None;
        match from_api.recv() {
            Ok(vmm_request) => {
                use api_server::VmmAction::*;
                let action_request = *vmm_request;

                // Also consume the API event. This is safe since communication
                // between this thread and the API thread is synchronous.
                let _ = api_event_fd.read().map_err(|e| {
                    error!("VMM: Failed to read the API event_fd: {}", e);
                    process::exit(i32::from(vmm::FC_EXIT_CODE_GENERIC_ERROR));
                });

                let response = match action_request {
                    /////////////////////////////////////////
                    // Supported operations allowed pre-boot.
                    ConfigureBootSource(boot_source_body) => vm_resources
                        .set_boot_source(boot_source_body)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| VmmActionError::BootSource(ErrorKind::User, e)),
                    ConfigureLogger(logger_description) => vmm_config::logger::init_logger(
                        logger_description,
                        crate_version!().to_string(),
                    )
                    .map(|_| api_server::VmmData::Empty)
                    .map_err(|e| VmmActionError::Logger(ErrorKind::User, e)),
                    GetVmConfiguration => Ok(api_server::VmmData::MachineConfiguration(
                        vm_resources.vm_config().clone(),
                    )),
                    InsertBlockDevice(block_device_config) => vm_resources
                        .set_block_device(block_device_config)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| e.into()),
                    InsertNetworkDevice(netif_body) => vm_resources
                        .set_net_device(netif_body)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| e.into()),
                    SetVsockDevice(vsock_cfg) => {
                        vm_resources.set_vsock_device(vsock_cfg);
                        Ok(api_server::VmmData::Empty)
                    }
                    SetVmConfiguration(machine_config_body) => vm_resources
                        .set_vm_config(machine_config_body)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| e.into()),
                    UpdateBlockDevicePath(drive_id, path_on_host) => vm_resources
                        .update_block_device_path(drive_id, path_on_host)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| e.into()),
                    UpdateNetworkInterface(netif_update) => vm_resources
                        .update_net_rate_limiters(netif_update)
                        .map(|_| api_server::VmmData::Empty)
                        .map_err(|e| e.into()),
                    StartMicroVm => {
                        vmm::builder::build_microvm(&vm_resources, epoll_context, seccomp_level)
                            .map(|vmm| {
                                built_vmm = Some(vmm);
                                api_server::VmmData::Empty
                            })
                    }

                    ///////////////////////////////////
                    // Operations not allowed pre-boot.
                    FlushMetrics => Err(VmmActionError::Logger(
                        ErrorKind::User,
                        vmm_config::logger::LoggerConfigError::FlushMetrics(
                            "Cannot flush metrics before starting microVM.".to_string(),
                        ),
                    )),
                    RescanBlockDevice(_) => {
                        Err(vmm_config::drive::DriveError::OperationNotAllowedPreBoot.into())
                    }
                    SendCtrlAltDel => Err(vmm::error::VmmActionError::OperationNotSupportedPreBoot),
                };
                // Send back the result.
                to_api
                    .send(Box::new(response))
                    .map_err(|_| ())
                    .expect("one-shot channel closed");
                // If microVM was successfully started, return the Vmm.
                if let Some(vmm) = built_vmm {
                    break vmm;
                }
            }
            Err(RecvError) => {
                panic!("The channel's sending half was disconnected. Cannot receive data.");
            }
        };
    }
}

/// Handles the control event.
/// Receives and runs the Vmm action and sends back a response.
/// Provides program exit codes on errors.
fn vmm_control_event(
    controller: &mut VmmController,
    api_event_fd: &EventFd,
    from_api: &Receiver<VmmRequest>,
    to_api: &Sender<VmmResponse>,
) -> Result<(), u8> {
    use vmm::{ErrorKind, VmmActionError};

    api_event_fd.read().map_err(|e| {
        error!("VMM: Failed to read the API event_fd: {}", e);
        vmm::FC_EXIT_CODE_GENERIC_ERROR
    })?;

    match from_api.try_recv() {
        Ok(vmm_request) => {
            use api_server::VmmAction::*;
            let action_request = *vmm_request;
            let response = match action_request {
                ///////////////////////////////////
                // Supported operations allowed post-boot.
                FlushMetrics => controller
                    .flush_metrics()
                    .map(|_| api_server::VmmData::Empty),
                GetVmConfiguration => Ok(api_server::VmmData::MachineConfiguration(
                    controller.vm_config().clone(),
                )),
                RescanBlockDevice(drive_id) => controller
                    .rescan_block_device(&drive_id)
                    .map(|_| api_server::VmmData::Empty),
                #[cfg(target_arch = "x86_64")]
                SendCtrlAltDel => controller
                    .send_ctrl_alt_del()
                    .map(|_| api_server::VmmData::Empty),
                UpdateBlockDevicePath(drive_id, path_on_host) => controller
                    .update_block_device_path(drive_id, path_on_host)
                    .map(|_| api_server::VmmData::Empty),
                UpdateNetworkInterface(netif_update) => controller
                    .update_net_rate_limiters(netif_update)
                    .map(|_| api_server::VmmData::Empty),

                ///////////////////////////////////
                // Operations not allowed post-boot.
                ConfigureBootSource(_) => Err(VmmActionError::BootSource(
                    ErrorKind::User,
                    vmm_config::boot_source::BootSourceConfigError::UpdateNotAllowedPostBoot,
                )),
                ConfigureLogger(_) => Err(VmmActionError::Logger(
                    ErrorKind::User,
                    vmm_config::logger::LoggerConfigError::InitializationFailure(
                        "Cannot initialize logger after boot.".to_string(),
                    ),
                )),
                InsertBlockDevice(_) => {
                    Err(vmm_config::drive::DriveError::UpdateNotAllowedPostBoot.into())
                }
                InsertNetworkDevice(_) => {
                    Err(vmm_config::net::NetworkInterfaceError::UpdateNotAllowedPostBoot.into())
                }
                SetVsockDevice(_) => Err(VmmActionError::VsockConfig(
                    ErrorKind::User,
                    vmm_config::vsock::VsockError::UpdateNotAllowedPostBoot,
                )),
                SetVmConfiguration(_) => {
                    Err(vmm_config::machine_config::VmConfigError::UpdateNotAllowedPostBoot.into())
                }

                StartMicroVm => Err(vmm::error::StartMicrovmError::MicroVMAlreadyRunning.into()),
            };
            // Send back the result.
            to_api
                .send(Box::new(response))
                .map_err(|_| ())
                .expect("one-shot channel closed");
        }
        Err(TryRecvError::Empty) => {
            warn!("Got a spurious notification from api thread");
        }
        Err(TryRecvError::Disconnected) => {
            panic!("The channel's sending half was disconnected. Cannot receive data.");
        }
    };
    Ok(())
}
