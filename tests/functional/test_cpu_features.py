from host_tools.network import SSHConnection
import re


def check_cpu_topology(test_microvm, expected_cpu_topology):
    """
    Different topologies can be tested the same way once the microvm is
    started. This is a wrapper function for calling lscpu and checking if the
    command returns the expected cpu topology.
    """
    ssh_connection = SSHConnection(test_microvm.slot.ssh_config)

    # Execute the lscpu command to check the guest topology
    _, stdout, stderr = ssh_connection.execute_command("lscpu")
    assert (stderr.read().decode("utf-8") == '')
    # Read Line by line the stdout of lscpu to check the relevant information
    # regarding the CPU topology
    while True:
        line = stdout.readline()
        if line != '':
            [key, value] = list(map(lambda x: x.strip(), line.split(':')))
            if key in expected_cpu_topology.keys():
                assert value == expected_cpu_topology[key],\
                    "%s does not have the expected value" % key
        else:
            break

    ssh_connection.close()


def test_1vcpu(test_microvm_with_ssh, network_config):
    test_microvm = test_microvm_with_ssh

    test_microvm.basic_config(vcpu_count=1, net_iface_count=0)
    """
    Sets up the microVM with 1 vCPUs, 256 MiB of RAM, 0 network ifaces and
    a root file system with the rw permission. The network interfaces is
    added after we get an unique MAC and IP.
    """
    test_microvm.basic_network_config(network_config)

    test_microvm.start()

    expected_cpu_topology = {
        "CPU(s)": "1",
        "On-line CPU(s) list": "0",
        "Thread(s) per core": "1",
        "Core(s) per socket": "1",
        "Socket(s)": "1",
        "NUMA node(s)": "1"
    }
    check_cpu_topology(test_microvm, expected_cpu_topology)


def test_2vcpu_ht_disabled(test_microvm_with_ssh, network_config):
    test_microvm = test_microvm_with_ssh

    test_microvm.basic_config(vcpu_count=2, ht_enable=False, net_iface_count=0)
    """
    Sets up the microVM with 2 vCPUs, 256 MiB of RAM, 0 network ifaces and
    a root file system with the rw permission. The network interfaces is
    added after we get an unique MAC and IP.
    """

    test_microvm.basic_network_config(network_config)

    test_microvm.start()

    expected_cpu_topology = {
        "CPU(s)": "2",
        "On-line CPU(s) list": "0,1",
        "Thread(s) per core": "1",
        "Core(s) per socket": "2",
        "Socket(s)": "1",
        "NUMA node(s)": "1"
    }
    check_cpu_topology(test_microvm, expected_cpu_topology)


def test_brand_string(test_microvm_with_ssh, network_config):
    """
    For Intel CPUs, the guest brand string should be:
        Intel(R) Xeon(R) Processor @ {host frequency}
    where {host frequency} is the frequency reported by the host CPUID
    (e.g. 4.01GHz)

    For non-Intel CPUs, the host and guest brand strings should be the same
    """
    cif = open('/proc/cpuinfo', 'r')
    host_brand_string = None
    while True:
        line = cif.readline()
        if line == '':
            break
        mo = re.search("^model name\\s+:\\s+(.+)$", line)
        if mo:
            host_brand_string = mo.group(1)
            break
    cif.close()
    assert(host_brand_string is not None)

    test_microvm = test_microvm_with_ssh
    test_microvm.basic_config(vcpu_count=1, net_iface_count=0)
    test_microvm.basic_network_config(network_config)
    test_microvm.start()

    ssh_connection = SSHConnection(test_microvm.slot.ssh_config)

    guest_cmd = "cat /proc/cpuinfo | grep 'model name' | head -1"
    _, stdout, stderr = ssh_connection.execute_command(guest_cmd)
    assert (stderr.read().decode("utf-8") == '')

    mo = re.search("^model name\\s+:\\s+(.+)$", stdout.readline().rstrip())
    assert(not (mo is None))
    guest_brand_string = mo.group(1)
    assert(len(guest_brand_string) > 0)

    expected_guest_brand_string = host_brand_string
    if host_brand_string.startswith("Intel"):
        expected_guest_brand_string = "Intel(R) Xeon(R) Processor"
        mo = re.search("[.0-9]+[MG]Hz", host_brand_string)
        if mo:
            expected_guest_brand_string += " @ " + mo.group(0)

    assert(guest_brand_string == expected_guest_brand_string)


