# Firecracker Read Me

## What is Firecracker

Firecracker is a new virtualization technology that enables customers to deploy
lightweight *micro* Virtual Machines or microVMs. Firecracker microVMs combine
the security and workload isolation properties of traditional VMs with the speed
and resource efficiency enabled by containers. MicroVMs can initiate user-space
code execution in less than 150ms, have a footprint of less than 32 MiB of
memory, and provide a secure, trusted environment for multi-tenant services.
Customers can create microVMs with any combination of vCPU and memory to match
their application requirements.

MicroVMs are created and managed by the Firecracker process, which implements a
virtual machine manager based on Linux's Kernel-based Virtual Machine (KVM), the
state of art for Linux virtualization. Firecracker provides the minimal required
device emulation to the guest operating system while excluding non-essential
functionality to enable faster startup time and a reduced memory footprint. The
Firecracker process also provides a control API, enforces microVM sandboxing,
and handles resource rate limiting for microVMs.

## What's Included

Firecracker consists of a single micro Virtual Machine Manager binary that will
spawn a RESTful API endpoint when started. The API supported by the current version
can be found at `api_server/swagger/firecracker.yaml`.

The **API endpoint** can be used to:

- Configure the microvm by:
  - Change the number of vCPUs (the default is 1)
  - Change the memory size (the default is 128 MiB)
  - Set a CPU template (the only available template is T2 for now)
  - Enable/Disable hyperthreading (by default hyperthreading is disabled).
    The host needs to be modified before starting Firecracker as this flag
    only changes the topology inside the microvm.
- Add one or more network interfaces to the microVM. Firecracker is mapping
  an existing host file as a VirtIO block device into the microVM.
- Add one or more read/write disks (file-backed block devices) to the microVM.
- Configure the logging system (i.e. path on host for log file, log level, etc).
- Configure rate limiters for VirtiIO devices which can limit the bandwidth, ops/s
  or both.
- Start the microVM using a given kernel image, root file system and boot arguments.
- Stop the microVM.

**Additional capabilities**:
- Emulated keyboard (i8042) and serial console (UART). The microVM serial
  console input and output are connected to those of the Firecracker process
  (this allows direct console access to the guest OS).
- Default demand fault paging & CPU oversubscription.

## Performance

### Manual one-time benchmarking and stress testing:

- An `iperf` network test (TCP, send-only, 1500 MTU, single core) from the
  microVM to the host has reached **8 Gbps** on an `i3.metal` EC2 instance. The
  test fully saturated 1 CPU core within the microVM, while the Firecracker
  thread running VirtIO Net emulation consumed another 75% of a host core.
- A `dd` storage test (writing large chunks of data to a file-backed block
  device stored in a RAM-disk, single-core) ran at 1 **GB/s**. The test fully
  saturated 1 CPU core within also microVM, while the Firecracker thread running
  VirtIO Block emulation consumed another 50% of a host core.
- At least **2000** Firecracker microVMs have been started on a single host
  (each with 1 vCPu core, and one TUN/TAP device), and have been stable under
  CPU and network I/O stress conditions, on an `i3.metal` EC2 instance.

### Integration testing:

- The boot-time SLA is enforced in `tests/performance/test-boottime.py`

## Getting Started

### Get or Build the Firecracker Binary

You can grab the latest Firecracker binary from the release S3 bucket.
You can request access to the S3 bucket by opening an issue with the label
"Support: Access Request".

If you want to build it from source, you'll need add the Rust `musl` toolchain:

``` bash
rustup target add x86_64-unknown-linux-musl
cargo build --release
```
The binary is located under target/x86_64-unknown-linux-musl/release/firecracker.

### Secure a Host with KVM Access

To build, test, or run, Firecracker requires a host with a modern version of KVM
(Linux kernel 4.14+) running on physical hardware (or a virtual machine with
nested virtualization enabled).

Firecracker needs rw access to `/dev/kvm`. You can grant these rights, e.g., to
all users, with: `sudo chmod a+rw /dev/kvm`.

### Start Firecracker

Execute the Firecracker binary, whose single argument is the API unix socket
name.
```bash
./firecracker --api-sock /tmp/dummy.socket
```

If the api socket is not specified at startup, Firecracker will create
/tmp/firecracker.socket.

### Configure the MicroVM

The MicroVM is configured via the `machine-config/` API resource.  
Example with cURL:
```bash
curl --unix-socket /tmp/firecracker.socket -i \
     -X PUT "http://localhost/machine-config" \
     -H "accept: application/json" \
     -H "Content-Type: application/json" \
     -d "{
            \"vcpu_count\": 6,
            \"mem_size_mib\": 3906,
            \"cpu_template\": \"T2\",
            \"ht_enabled\": true,
        }"
```


### Provision Network / Storage Resources

Firecracker expects network interfaces and drives to be created beforehand and
passed by name. Ensure Firecracker will have the required permissions to open
these resources.

For example, if using a TUN/TAP device, you will need to create it beforehand,
and then call the `/network-interfaces` API resource with its name:
```bash
sudo ip tuntap add name vmtap33 mode tap
sudo ifconfig vmtap33 192.168.241.1/24 up

curl --unix-socket /tmp/firecracker.socket -i \
     -X PUT "http://localhost/network-interfaces/1" \
     -H "accept: application/json" \
     -H "Content-Type: application/json" \
     -d "{
            \"iface_id\": \"1\",
            \"host_dev_name\": \"vmtap33\",
            \"guest_mac\": \"06:00:00:00:00:01\",
            \"state\": \"Attached\"
        }"
```

### Select the Guest Kernel and RootFS

To run a guest OS within a Firecracker microVMs, you will need have:

- **A guest kernel image** that boots and runs with Firecracker's minimal/VirtIO
  device model. Pass this via the `/boot-source` API resource.
- **A guest root file system** that boots with that kernel. You'll pass this as
  a bootable block device to Firecracker via the `/drives` API resource.

### Power-On the MicroVM

Simply issue the `InstanceStart` action to the `/actions` API resource.
This is an asynchronous API request. You can check the response with a get
on the same path.
```bash
# Start the Firecracker MicroVM
curl --unix-socket /tmp/firecracker.socket -i \
     -X PUT "http://localhost/actions/start" \
     -H  "accept: application/json" \
     -H  "Content-Type: application/json" \
     -d "{
            \"action_id\": \"start\",
            \"action_type\": \"InstanceStart\"
         }"

# Wait some time for the MicroVM to boot
sleep 0.5

# Get the response of starting the MicroVM
curl --unix-socket /tmp/firecracker.socket -i \
     -X GET "http://localhost/actions/start" \
     -H "accept: application/json"

```

### Notes

1. It is the user's responsibility to make sure that the same backing file is
   not added as a read-write block device to multiple Firecracker instances. A
   file can be safely added as a read-only block device to multiple Firecracker
   instances.
1. Firecracker is started without the serial console for performance reasons.
   You can use the following boot_args if you need the serial console:
   `console=ttyS0 noapic reboot=k panic=1 pci=off nomodules`  
1. Firecracker uses default values for the following parameters:
    1. Kernel Command Line:
       `noapic reboot=k panic=1 pci=off nomodules 8250.nr_uarts=0`. This can be
       changed via the `/boot-source`.
    1. Number of vCPUs: 1. Default Memory Size: 128 MiB. Hyperthreading is 
       disabled. CPU Template: None.
    1. Unix domain socket: `/tmp/firecracker.socket`.
1. Firecracker links the microVM serial console output to its stdout, and its
   stdin to the microVM serial console input. Therefore, you can interact with
   the microVM guest in the screen session.

### Caveats

1. The unix domain socket is not deleted when Firecracker is stopped.
   You have to remove it yourself after stopping the Firecracker process.
1. Firecracker doesn't yet emulate a power management device. This means that
   any shutdown/poweroff/halt commands issued by the guest OS will not work as
   intended.

### Getting Started Code Example

For a full example, you can take a look at the `test_api_happy_start` test in
[tests/functional/test_api.py](tests/functional/test_api.py), and at the
`basic_config` method of the `Microvm` class in
[tests/microvm.py](tests/microvm.py).

