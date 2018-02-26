#[macro_use(crate_version, crate_authors)]
extern crate clap;

extern crate api_server;
extern crate devices;
extern crate sys_util;
extern crate vmm;

use clap::{App, Arg};
use std::path::PathBuf;
use std::sync::mpsc::channel;

use api_server::ApiServer;
use sys_util::{syslog, EventFd};
use vmm::machine::MachineCfg;
use vmm::device_config::BlockDeviceConfig;

fn main() {
    if let Err(e) = syslog::init() {
        println!("failed to initialize syslog: {:?}", e);
        return;
    }

    let cmd_arguments = App::new("firecracker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Launch a microvm.")
        .arg(
            Arg::with_name("vmm_no_api")
                .long("vmm-no-api")
                .help("Start vmm by default, no API thread")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("api_sock")
                .long("api-sock")
                .help("Path to unix domain socket used by the API")
                .default_value("/tmp/firecracker.socket")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("kernel_path")
                .short("k")
                .long("kernel-path")
                .help("The kernel's file path (vmlinux.bin)")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("kernel_cmdline")
                .long("kernel-cmdline")
                .help("The kernel's command line")
                .default_value("console=ttyS0 noapic reboot=k panic=1 pci=off nomodules")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mem_size")
                .long("mem-size")
                .default_value("128")
                .help("Virtual Machine Memory Size in MiB")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vcpu_count")
                .long("vcpu-count")
                .default_value("1")
                .help("Number of VCPUs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("root_blk_file")
                .short("r")
                .long("root-blk")
                .help("File to serve as root block device [enabled only with --vmm-no-api]")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("host_ip")
                .long("host-ip")
                .help("IPv4 address of the host interface")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("subnet_mask")
                .long("subnet-mask")
                .default_value("255.255.255.0")
                .help("Subnet mask for the IP address of host interface")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("vsock_guest_cid")
                .long("vsock-guest-cid")
                .help("The guest CID for the virtio-vhost-vsock device")
                .takes_value(true)
        )
        /*
        The mac address option is not currently implemented; the L2 addresses for both the
        host interface and the guest interface take some implicit (possibly random) values
        .arg(
            Arg::with_name("mac_address")
                .long("mac-addr")
                .help("MAC address for the VM")
                .takes_value(true)
        )*/
        .get_matches();

    let vmm_no_api = cmd_arguments.is_present("vmm_no_api");

    let cfg = parse_args(&cmd_arguments);
    let (to_vmm, from_api) = channel();

    // TODO: vmm_no_api is for integration testing, need to find a more pretty solution
    if vmm_no_api {
        let mut vmm = vmm::Vmm::new(
            cfg,
            EventFd::new().expect("cannot create eventFD"),
            from_api,
        ).expect("cannot create VMM");
        // This is a temporary fix. Block devices should be added via http requests.
        // With the command line, we can only add one device, with default to root block device.
        if cmd_arguments.is_present("root_blk_file") {
            let root_block_device = BlockDeviceConfig {
                path_on_host: PathBuf::from(cmd_arguments.value_of("root_blk_file").unwrap()),
                is_root_device: true,
                drive_id: String::from("1"),
            };
            vmm.put_block_device(root_block_device).expect("cannot add root block device.");
        }

        if cmd_arguments.is_present("host_ip") {
            let host_ipv4_address = cmd_arguments
                .value_of("host_ip")
                .unwrap()
                .parse()
                .expect("bad host ip");

            let host_netmask = cmd_arguments
                .value_of("subnet_mask")
                .unwrap()
                .parse()
                .expect("bad subnet mask");

            use api_server::request::sync::DeviceState;
            use api_server::request::sync::NetworkInterfaceBody;

            let body = NetworkInterfaceBody {
                iface_id: String::from("0"),
                state: DeviceState::Attached,
                host_dev_name: String::from("vmtap%d"),
                host_ipv4_address,
                host_netmask,
                guest_mac: None,
            };

            use vmm::device_config::NetworkInterfaceConfig;

            let cfg =
                NetworkInterfaceConfig::try_from_body(body).expect("bad interface body definition");
            vmm.add_net_device(cfg);
        }

        vmm.boot_kernel().expect("cannot boot kernel");
        vmm.run_control().expect("VMM loop error!");
    } else {
        // safe to unwrap since api_sock has a default value
        let bind_path = cmd_arguments
            .value_of("api_sock")
            .map(|s| PathBuf::from(s))
            .unwrap();

        let server = ApiServer::new(to_vmm, 100).unwrap();
        let api_event_fd = server
            .get_event_fd_clone()
            .expect("cannot clone API eventFD");
        let _vmm_thread_handle = vmm::start_vmm_thread(cfg, api_event_fd, from_api);
        server.bind_and_run(bind_path).unwrap();
    }
}

fn parse_args(cmd_arguments: &clap::ArgMatches) -> MachineCfg {
    let kernel_path: Option<PathBuf> = cmd_arguments
        .value_of("kernel_path")
        .map(|s| PathBuf::from(s));

    //unwrap should not panic because kernel_cmdline has a default value
    let kernel_cmdline = String::from(cmd_arguments.value_of("kernel_cmdline").unwrap());

    let vcpu_count = match cmd_arguments
        .value_of("vcpu_count")
        .unwrap()
        .to_string()
        .parse::<u8>()
    {
        Ok(value) => value,
        Err(error) => {
            panic!("Invalid value for vcpu_count! {:?}", error);
        }
    };

    let mem_size = match cmd_arguments
        .value_of("mem_size")
        .unwrap()
        .to_string()
        .parse::<usize>()
    {
        Ok(value) => value,
        Err(error) => {
            panic!("Invalid value for mem_size! {:?}", error);
        }
    };

    let root_blk_file = cmd_arguments
        .value_of("root_blk_file")
        .map(|s| PathBuf::from(s));

    //fixme print some message when the Ipv4Addrs cannot be parsed
    let host_ip = cmd_arguments
        .value_of("host_ip")
        .map(|x| x.parse().unwrap());

    let subnet_mask = cmd_arguments
        .value_of("subnet_mask")
        .unwrap()
        .parse()
        .unwrap();

    let vsock_guest_cid = match cmd_arguments.value_of("vsock_guest_cid") {
        Some(cid) => match cid.parse::<u64>() {
            Ok(value) => Some(value),
            Err(error) => {
                panic!(
                    "Invalid parameter value for the vsock's guest CID! {:?}",
                    error
                );
            }
        },
        None => None,
    };

    MachineCfg::new(
        kernel_path,
        kernel_cmdline,
        vcpu_count,
        mem_size,
        root_blk_file,
        host_ip,
        subnet_mask,
        vsock_guest_cid,
    )
}
