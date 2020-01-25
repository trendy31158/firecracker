// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use api_server::{ApiRequest, ApiResponse, ApiServer};
use mmds::MMDS;
use polly::event_manager::{EventHandler, EventManager};
use polly::pollable::{Pollable, PollableOp, PollableOpBuilder};
use utils::eventfd::EventFd;
use vmm::controller::VmmController;
use vmm::rpc_interface::{PrebootApiController, RuntimeApiController};
use vmm::vmm_config::instance_info::InstanceInfo;

struct ApiServerAdapter {
    api_event_fd: EventFd,
    from_api: Receiver<ApiRequest>,
    to_api: Sender<ApiResponse>,
    controller: RuntimeApiController,
}

impl ApiServerAdapter {
    /// Runs the vmm to completion, while any arising control events are deferred
    /// to a `RuntimeApiController`.
    fn run_microvm(
        api_event_fd: EventFd,
        from_api: Receiver<ApiRequest>,
        to_api: Sender<ApiResponse>,
        vmm_controller: VmmController,
        event_manager: &mut EventManager,
    ) {
        let api_adapter = Arc::new(Mutex::new(Self {
            api_event_fd,
            from_api,
            to_api,
            controller: RuntimeApiController(vmm_controller),
        }));
        event_manager
            .register(api_adapter.clone())
            .expect("Cannot register the api event to the event manager.");
        loop {
            event_manager
                .run()
                .expect("EventManager events driver fatal error");
        }
    }
}
impl EventHandler for ApiServerAdapter {
    /// Handle a read event (EPOLLIN).
    fn handle_read(&mut self, source: Pollable) -> Vec<PollableOp> {
        if source == self.api_event_fd.as_raw_fd() {
            let _ = self.api_event_fd.read();
            match self.from_api.try_recv() {
                Ok(api_request) => {
                    let response = self.controller.handle_request(*api_request);
                    // Send back the result.
                    self.to_api
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
        } else {
            error!("Spurious EventManager event for handler: ApiServerAdapter");
        }
        vec![]
    }

    fn init(&self) -> Vec<PollableOp> {
        vec![PollableOpBuilder::new(self.api_event_fd.as_raw_fd())
            .readable()
            .register()]
    }
}

pub fn run_with_api(
    seccomp_level: u32,
    config_json: Option<String>,
    bind_path: PathBuf,
    instance_info: InstanceInfo,
    start_time_us: Option<u64>,
    start_time_cpu_us: Option<u64>,
) {
    // FD to notify of API events.
    let api_event_fd = EventFd::new(libc::EFD_NONBLOCK).expect("Cannot create API Eventfd.");
    // Channels for both directions between Vmm and Api threads.
    let (to_vmm, from_api) = channel();
    let (to_api, from_vmm) = channel();

    // MMDS only supported with API.
    let mmds_info = MMDS.clone();
    let api_shared_info = Arc::new(RwLock::new(instance_info));
    let vmm_shared_info = api_shared_info.clone();
    let to_vmm_event_fd = api_event_fd.try_clone().unwrap();

    // Start the separate API thread.
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
            .bind_and_run(bind_path, start_time_us, start_time_cpu_us, seccomp_level)
            {
                Ok(_) => (),
                Err(api_server::Error::Io(inner)) => match inner.kind() {
                    std::io::ErrorKind::AddrInUse => panic!(
                        "Failed to open the API socket: {:?}",
                        api_server::Error::Io(inner)
                    ),
                    _ => panic!(
                        "Failed to communicate with the API socket: {:?}",
                        api_server::Error::Io(inner)
                    ),
                },
                Err(eventfd_err @ api_server::Error::Eventfd(_)) => {
                    panic!("Failed to open the API socket: {:?}", eventfd_err)
                }
            }
        })
        .expect("API thread spawn failed.");

    // The driving epoll engine.
    let mut epoll_context = vmm::EpollContext::new().expect("Cannot create the epoll context.");
    // The event manager to replace EpollContext.
    let mut event_manager = EventManager::new().expect("Unable to create EventManager");

    // Create the firecracker metrics object responsible for periodically printing metrics.
    let firecracker_metrics = Arc::new(Mutex::new(super::metrics::PeriodicMetrics::new()));
    event_manager
        .register(firecracker_metrics.clone())
        .expect("Cannot register the metrics event to the event manager.");

    let firecracker_version = crate_version!().to_string();
    // Configure, build and start the microVM.
    let (vm_resources, vmm) = match config_json {
        Some(json) => super::build_microvm_from_json(
            seccomp_level,
            &mut epoll_context,
            &mut event_manager,
            firecracker_version,
            json,
        ),
        None => PrebootApiController::build_microvm_from_requests(
            seccomp_level,
            &mut epoll_context,
            &mut event_manager,
            firecracker_version,
            || {
                let req = from_api
                    .recv()
                    .expect("The channel's sending half was disconnected. Cannot receive data.");
                // Also consume the API event along with the message. It is safe to unwrap()
                // since communication between this thread and the API thread is synchronous.
                api_event_fd
                    .read()
                    .expect("VMM: Failed to read the API event_fd");
                *req
            },
            |response| {
                to_api
                    .send(Box::new(response))
                    .expect("one-shot channel closed")
            },
        ),
    };

    // Start the metrics.
    firecracker_metrics
        .lock()
        .expect("Metrics lock poisoned.")
        .start(super::metrics::WRITE_METRICS_PERIOD_MS);

    // Update the api shared instance info.
    api_shared_info.write().unwrap().started = true;

    // TODO: remove this when last epoll_context user is migrated to EventManager.
    let epoll_context = Arc::new(Mutex::new(epoll_context));
    event_manager.register(epoll_context).unwrap();

    ApiServerAdapter::run_microvm(
        api_event_fd,
        from_api,
        to_api,
        VmmController::new(vm_resources, vmm),
        &mut event_manager,
    );
}
