# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests for the CPU topology emulation feature."""

from pathlib import Path
import platform
import os
import shutil
import re
import pytest
import pandas as pd

from conftest import _test_images_s3_bucket
from framework import utils
from framework.artifacts import ArtifactCollection, ArtifactSet, NetIfaceConfig
from framework.matrix import TestMatrix, TestContext
from framework.builder import MicrovmBuilder
from framework.defs import SUPPORTED_KERNELS
import framework.utils_cpuid as cpuid_utils
import host_tools.network as net_tools

PLATFORM = platform.machine()


def _check_cpuid_x86(test_microvm, expected_cpu_count, expected_htt):
    expected_cpu_features = {
        "cpu count": "{} ({})".format(hex(expected_cpu_count), expected_cpu_count),
        "CLFLUSH line size": "0x8 (8)",
        "hypervisor guest status": "true",
        "hyper-threading / multi-core supported": expected_htt,
    }

    cpuid_utils.check_guest_cpuid_output(
        test_microvm, "cpuid -1", None, "=", expected_cpu_features
    )


def _check_cpu_features_arm(test_microvm):
    if cpuid_utils.get_instance_type() == "m6g.metal":
        expected_cpu_features = {
            "Flags": "fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp "
            "asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs",
        }
    else:
        expected_cpu_features = {
            "Flags": "fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp "
            "asimdhp cpuid asimdrdm jscvt fcma lrcpc dcpop sha3 sm3 sm4 asimddp "
            "sha512 asimdfhm dit uscat ilrcpc flagm ssbs",
        }

    cpuid_utils.check_guest_cpuid_output(
        test_microvm, "lscpu", None, ":", expected_cpu_features
    )


@pytest.mark.skipif(PLATFORM != "x86_64", reason="CPUID is only supported on x86_64.")
@pytest.mark.parametrize(
    "num_vcpus",
    [1, 2, 16],
)
@pytest.mark.parametrize(
    "htt",
    [True, False],
)
def test_cpuid(test_microvm_with_api, network_config, num_vcpus, htt):
    """
    Check the CPUID for a microvm with the specified config.

    @type: functional
    """
    vm = test_microvm_with_api
    vm.spawn()
    vm.basic_config(vcpu_count=num_vcpus, smt=htt)
    _tap, _, _ = vm.ssh_network_config(network_config, "1")
    vm.start()
    _check_cpuid_x86(vm, num_vcpus, "true" if num_vcpus > 1 else "false")


@pytest.mark.skipif(
    PLATFORM != "aarch64",
    reason="The CPU features on x86 are tested as part of the CPU templates.",
)
def test_cpu_features(test_microvm_with_api, network_config):
    """
    Check the CPU features for a microvm with the specified config.

    @type: functional
    """
    vm = test_microvm_with_api
    vm.spawn()
    vm.basic_config()
    _tap, _, _ = vm.ssh_network_config(network_config, "1")
    vm.start()
    _check_cpu_features_arm(vm)


@pytest.mark.skipif(
    PLATFORM != "x86_64", reason="The CPU brand string is masked only on x86_64."
)
def test_brand_string(test_microvm_with_api, network_config):
    """
    Ensure good formatting for the guest brand string.

    * For Intel CPUs, the guest brand string should be:
        Intel(R) Xeon(R) Processor @ {host frequency}
    where {host frequency} is the frequency reported by the host CPUID
    (e.g. 4.01GHz)
    * For AMD CPUs, the guest brand string should be:
        AMD EPYC
    * For other CPUs, the guest brand string should be:
        ""

    @type: functional
    """
    cif = open("/proc/cpuinfo", "r", encoding="utf-8")
    host_brand_string = None
    while True:
        line = cif.readline()
        if line == "":
            break
        mo = re.search("^model name\\s+:\\s+(.+)$", line)
        if mo:
            host_brand_string = mo.group(1)
    cif.close()
    assert host_brand_string is not None

    test_microvm = test_microvm_with_api
    test_microvm.spawn()

    test_microvm.basic_config(vcpu_count=1)
    _tap, _, _ = test_microvm.ssh_network_config(network_config, "1")
    test_microvm.start()

    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)

    guest_cmd = "cat /proc/cpuinfo | grep 'model name' | head -1"
    _, stdout, stderr = ssh_connection.execute_command(guest_cmd)
    assert stderr.read() == ""

    line = stdout.readline().rstrip()
    mo = re.search("^model name\\s+:\\s+(.+)$", line)
    assert mo
    guest_brand_string = mo.group(1)
    assert guest_brand_string

    cpu_vendor = cpuid_utils.get_cpu_vendor()
    expected_guest_brand_string = ""
    if cpu_vendor == cpuid_utils.CpuVendor.AMD:
        expected_guest_brand_string += "AMD EPYC"
    elif cpu_vendor == cpuid_utils.CpuVendor.INTEL:
        expected_guest_brand_string = "Intel(R) Xeon(R) Processor"
        mo = re.search("[.0-9]+[MG]Hz", host_brand_string)
        if mo:
            expected_guest_brand_string += " @ " + mo.group(0)

    assert guest_brand_string == expected_guest_brand_string


# Some MSR values should not be checked since they can change at Guest runtime.
# Current exceptions:
#   * FS and GS change on task switch and arch_prctl.
#   * TSC is different for each Guest.
#   * MSR_{C, L}STAR used for SYSCALL/SYSRET; can be different between guests.
#   * MSR_IA32_SYSENTER_E{SP, IP} used for SYSENTER/SYSEXIT; same as above.
#   * MSR_KVM_{WALL, SYSTEM}_CLOCK addresses for struct pvclock_* can be different.
#   * MSR_IA32_TSX_CTRL is not available to read/write via KVM (known limitation).
#
# More detailed information about MSRs can be found in the Intel® 64 and IA-32
# Architectures Software Developer’s Manual - Volume 4: Model-Specific Registers
# Check `arch_gen/src/x86/msr_idex.rs` and `msr-index.h` in upstream Linux
# for symbolic definitions.
# fmt: off
MSR_EXCEPTION_LIST = [
    "0x10",        # MSR_IA32_TSC
    "0x11",        # MSR_KVM_WALL_CLOCK
    "0x12",        # MSR_KVM_SYSTEM_TIME
    "0x122",       # MSR_IA32_TSX_CTRL
    "0x175",       # MSR_IA32_SYSENTER_ESP
    "0x176",       # MSR_IA32_SYSENTER_EIP
    "0x6e0",       # MSR_IA32_TSCDEADLINE
    "0xc0000082",  # MSR_LSTAR
    "0xc0000083",  # MSR_CSTAR
    "0xc0000100",  # MSR_FS_BASE
    "0xc0000101",  # MSR_GS_BASE
]
# fmt: on


@pytest.mark.skipif(
    PLATFORM != "x86_64", reason="CPU features are masked only on x86_64."
)
@pytest.mark.skipif(
    cpuid_utils.get_cpu_vendor() != cpuid_utils.CpuVendor.INTEL,
    reason="CPU templates are only available on Intel.",
)
@pytest.mark.skipif(
    utils.get_kernel_version(level=1) not in SUPPORTED_KERNELS,
    reason=f"Supported kernels are {SUPPORTED_KERNELS}",
)
@pytest.mark.parametrize("cpu_template", ["T2S"])
@pytest.mark.timeout(900)
@pytest.mark.nonci
def test_cpu_rdmsr(bin_cloner_path, network_config, cpu_template):
    """
    Test MSRs that are available to the Guest.

    This test boots a Firecracker uVM and tries to read a set of MSRs from the guest.
    The Guest MSR list is compared against a list of MSRs that are expected when running
    on a particular host kernel and with a particular Guest CPU template.
    The list is different for each kernel version because Firecracker relies on
    MSR emulation provided by KVM. If KVM emulation changes, then the MSR list
    available to the guest might change also.
    The list is also dependant on CPUID (CPU templates) since some MSRs are not available
    if CPUID features are disabled.
    Lastly, this tests also checks for MSR values against the baseline. This helps validate
    that defaults have not changed due to emulation implementation changes in the kernel.

    TODO: This only validates T2S templates. Since T2 and C3 did not set the
    ARCH_CAPABILITIES MSR, the value of that MSR is different between different
    host CPU types (see Github PR #3066). So we can either:
        * add an exceptions for different template types when checking values
        * deprecate T2 and C3 since they are somewhat broken

    @type: functional
    """

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    # Testing matrix:
    # - Guest kernel: Linux 4.14 & Linux 5.10
    # - Rootfs: Ubuntu 18.04 with msr-tools package installed
    # - Microvm: 1vCPU with 1024 MB RAM
    microvm_artifacts = ArtifactSet(artifacts.microvms(keyword="1vcpu_1024mb"))
    kernel_artifacts = ArtifactSet(artifacts.kernels())
    disk_artifacts = ArtifactSet(artifacts.disks(keyword="bionic-msrtools"))
    assert len(disk_artifacts) == 1

    test_context = TestContext()
    test_context.custom = {
        "builder": MicrovmBuilder(bin_cloner_path),
        "network_config": network_config,
        "cpu_template": cpu_template,
    }
    test_matrix = TestMatrix(
        context=test_context,
        artifact_sets=[microvm_artifacts, kernel_artifacts, disk_artifacts],
    )
    test_matrix.run_test(_test_cpu_rdmsr)


def _test_cpu_rdmsr(context):
    vm_builder = context.custom["builder"]
    cpu_template = context.custom["cpu_template"]
    root_disk = context.disk.copy()

    vm_instance = vm_builder.build(
        kernel=context.kernel,
        disks=[root_disk],
        ssh_key=context.disk.ssh_key(),
        config=context.microvm,
        cpu_template=cpu_template,
    )
    test_microvm = vm_instance.vm
    test_microvm.start()

    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)
    ssh_connection.scp_file(
        "../resources/tests/msr/msr_reader.sh", "/bin/msr_reader.sh"
    )
    _, stdout, stderr = ssh_connection.execute_command("/bin/msr_reader.sh")
    assert stderr.read() == ""

    # Load results read from the microvm
    microvm_df = pd.read_csv(stdout)

    # Load baseline
    # Baselines are taken by running `msr_reader.sh` on:
    #  * host running kernel 4.14 and guest 4.14 with the `bionic-msrtools` rootfs
    #  * host running kernel 4.14 and guest 5.10 with the `bionic-msrtools` rootfs
    baseline_df = pd.read_csv(
        f"../resources/tests/msr/msr_list_{cpu_template}_{utils.get_kernel_version(level=1)}.csv"
    )

    # We first want to see if the same set of MSRs are exposed in the microvm.
    # Drop the VALUE columns and compare the 2 dataframes.
    impl_diff = pd.concat(
        [microvm_df.drop(columns="VALUE"), baseline_df.drop(columns="VALUE")],
        keys=["microvm", "baseline"],
    ).drop_duplicates(keep=False)
    assert impl_diff.empty, f"\n {impl_diff}"

    # Now drop the STATUS column to compare values for each MSR
    microvm_val_df = microvm_df.drop(columns="STATUS")
    baseline_val_df = baseline_df.drop(columns="STATUS")

    # pylint: disable=C0121
    microvm_val_df = microvm_val_df[
        microvm_val_df["MSR_ADDR"].isin(MSR_EXCEPTION_LIST) == False
    ]
    baseline_val_df = baseline_val_df[
        baseline_val_df["MSR_ADDR"].isin(MSR_EXCEPTION_LIST) == False
    ]

    # Also some MSRs are different based on Guest configuration and kernel used.
    # Guest Kernel 5.10 sets up some MSRs differently.
    if context.kernel.name() == "vmlinux-5.10.bin":
        guest_msrs_5_10 = {
            # See https://github.com/torvalds/linux/commit/1db2a6e1e29ff994443a9eef7cf3d26104c777a7
            "0x3a": "0x1",  # MSR_IA32_FEAT_CTL
            # See https://github.com/torvalds/linux/commit/229b969b3d38bc28bcd55841ee7ca9a9afb922f3
            "0x808": "0x10",  # IA32_X2APIC_TPR
            "0x80a": "0x10",  # IA32_X2APIC_PPR
            # `arch/x86/include/asm/irq_vectors.h` to see how LOCAL_TIMER_VECTOR changed
            "0x832": "0x400ec",  # IA32_X2APIC_LVT_TIMER
        }

        for key, value in guest_msrs_5_10.items():
            baseline_val_df.loc[baseline_val_df["MSR_ADDR"] == key, "VALUE"] = value

    # Compare values
    val_diff = pd.concat(
        [microvm_val_df, baseline_val_df], keys=["microvm", "baseline"]
    ).drop_duplicates(keep=False)
    assert val_diff.empty, f"\n {val_diff}"


# These names need to be consistent across the two parts of the snapshot-restore test
# that spans two instances (one that takes a snapshot and one that restores from it)
# fmt: off
SNAPSHOT_RESTORE_SHARED_NAMES = {
    "cpu_templates":                     [None, "T2S"],
    "snapshot_artifacts_root_dir_wrmsr": "snapshot_artifacts/wrmsr",
    "rootfs_fname":                      "rootfs_rw",
    "msr_reader_host_fname":             "../resources/tests/msr/msr_reader.sh",
    "msr_reader_guest_fname":            "/bin/msr_reader.sh",
    "msrs_before_fname":                 "msrs_before.txt",
    "msrs_after_fname":                  "msrs_after.txt",
    "cpuid_before_fname":                "cpuid_before.txt",
    "cpuid_after_fname":                 "cpuid_after.txt",
    "snapshot_fname":                    "vmstate",
    "mem_fname":                         "mem",
    # Testing matrix:
    # - Rootfs: Ubuntu 18.04 with msr-tools package installed
    # - Microvm: 1vCPU with 1024 MB RAM
    "disk_keyword":                      "bionic-msrtools",
    "microvm_keyword":                   "1vcpu_1024mb",
}
# fmt: on


def dump_msr_state_to_file(dump_fname, ssh_conn, shared_names):
    """
    Read MSR state via SSH and dump it into a file.
    """
    ssh_conn.scp_file(
        shared_names["msr_reader_host_fname"], shared_names["msr_reader_guest_fname"]
    )
    _, stdout, stderr = ssh_conn.execute_command(shared_names["msr_reader_guest_fname"])
    assert stderr.read() == ""

    with open(dump_fname, "w", encoding="UTF-8") as file:
        file.write(stdout.read())


def _test_cpu_wrmsr_snapshot(context):
    shared_names = context.custom["shared_names"]
    root_disk = context.disk.copy(file_name=shared_names["rootfs_fname"])
    vm_builder = context.custom["builder"]
    cpu_template = context.custom["cpu_template"]

    vm_instance = vm_builder.build(
        kernel=context.kernel,
        disks=[root_disk],
        ssh_key=context.disk.ssh_key(),
        config=context.microvm,
        diff_snapshots=True,
        cpu_template=cpu_template,
    )

    vm = vm_instance.vm
    vm.start()

    # Make MSR modifications
    ssh_connection = net_tools.SSHConnection(vm.ssh_config)

    msr_writer_host_fname = "../resources/tests/msr/msr_writer.sh"
    msr_writer_guest_fname = "/bin/msr_writer.sh"
    ssh_connection.scp_file(msr_writer_host_fname, msr_writer_guest_fname)

    wrmsr_input_host_fname = "../resources/tests/msr/wrmsr_list.txt"
    wrmsr_input_guest_fname = "/tmp/wrmsr_input.txt"
    ssh_connection.scp_file(wrmsr_input_host_fname, wrmsr_input_guest_fname)

    _, _, stderr = ssh_connection.execute_command(
        f"{msr_writer_guest_fname} {wrmsr_input_guest_fname}"
    )
    assert stderr.read() == ""

    # Dump MSR state to a file that will be published to S3 for the 2nd part of the test
    snapshot_artifacts_dir = (
        Path(shared_names["snapshot_artifacts_root_dir_wrmsr"])
        / context.kernel.base_name()
        / (cpu_template if cpu_template else "none")
    )
    os.makedirs(snapshot_artifacts_dir)

    msrs_before_fname = Path(snapshot_artifacts_dir) / shared_names["msrs_before_fname"]

    dump_msr_state_to_file(msrs_before_fname, ssh_connection, shared_names)

    # Take a snapshot
    vm.pause_to_snapshot(
        mem_file_path=shared_names["mem_fname"],
        snapshot_path=shared_names["snapshot_fname"],
        diff=True,
    )

    # Copy snapshot files to be published to S3 for the 2nd part of the test
    chroot_dir = vm.chroot()
    shutil.copyfile(
        Path(chroot_dir) / shared_names["mem_fname"],
        Path(snapshot_artifacts_dir) / shared_names["mem_fname"],
    )
    shutil.copyfile(
        Path(chroot_dir) / shared_names["snapshot_fname"],
        Path(snapshot_artifacts_dir) / shared_names["snapshot_fname"],
    )
    shutil.copyfile(
        root_disk.local_path(),
        Path(snapshot_artifacts_dir) / shared_names["rootfs_fname"],
    )


@pytest.mark.skipif(
    PLATFORM != "x86_64", reason="CPU features are masked only on x86_64."
)
@pytest.mark.skipif(
    cpuid_utils.get_cpu_vendor() != cpuid_utils.CpuVendor.INTEL,
    reason="CPU templates are only available on Intel.",
)
@pytest.mark.skipif(
    utils.get_kernel_version(level=1) not in SUPPORTED_KERNELS,
    reason=f"Supported kernels are {SUPPORTED_KERNELS}",
)
@pytest.mark.parametrize("cpu_template", SNAPSHOT_RESTORE_SHARED_NAMES["cpu_templates"])
@pytest.mark.timeout(900)
@pytest.mark.nonci
def test_cpu_wrmsr_snapshot(bin_cloner_path, cpu_template):
    """
    This is the first part of the test verifying
    that MSRs retain their values after restoring from a snapshot.

    This function makes MSR value modifications according to the
    ../resources/tests/msr/wrmsr_list.txt file.

    Before taking a snapshot, MSR values are dumped into a text file.
    After restoring from the snapshot on another instance, the MSRs are
    dumped again and their values are compared to previous.
    Some MSRs are not inherently supposed to retain their values, so they
    form an MSR exception list.

    This part of the test is responsible for taking a snapshot and publishing
    its files along with the `before` MSR dump.

    @type: functional
    """
    shared_names = SNAPSHOT_RESTORE_SHARED_NAMES

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    microvm_artifacts = ArtifactSet(
        artifacts.microvms(keyword=shared_names["microvm_keyword"])
    )
    kernel_artifacts = ArtifactSet(artifacts.kernels())
    disk_artifacts = ArtifactSet(artifacts.disks(keyword=shared_names["disk_keyword"]))
    assert len(disk_artifacts) == 1

    snapshot_artifacts_root_dir = shared_names["snapshot_artifacts_root_dir_wrmsr"]
    shutil.rmtree(snapshot_artifacts_root_dir, ignore_errors=True)
    os.makedirs(snapshot_artifacts_root_dir)

    test_context = TestContext()
    test_context.custom = {
        "builder": MicrovmBuilder(bin_cloner_path),
        "cpu_template": cpu_template,
        "shared_names": shared_names,
    }

    test_matrix = TestMatrix(
        context=test_context,
        artifact_sets=[microvm_artifacts, kernel_artifacts, disk_artifacts],
    )
    test_matrix.run_test(_test_cpu_wrmsr_snapshot)


def diff_msrs(before, after, column_to_drop):
    """
    Calculates and formats a diff between two MSR tables.
    """
    # Drop irrelevant column
    before_stripped = before.drop(column_to_drop, axis=1)
    after_stripped = after.drop(column_to_drop, axis=1)

    # Check that values in remaining columns are the same
    all_equal = (before_stripped == after_stripped).all(axis=None)

    # Arrange the diff as a side by side comparison of statuses
    not_equal = (before_stripped != after_stripped).any(axis=1)
    before_stripped.columns = ["MSR_ADDR", "Before"]
    after_stripped.columns = ["MSR_ADDR", "After"]
    diff = pd.merge(
        before_stripped[not_equal],
        after_stripped[not_equal],
        on=["MSR_ADDR", "MSR_ADDR"],
    ).to_string()

    # Return the diff or an empty string
    return diff if not all_equal else ""


def check_msr_values_are_equal(before_msrs_fname, after_msrs_fname, guest_kernel_name):
    """
    Checks that MSR statuses and values in the files are equal.
    """
    before = pd.read_csv(before_msrs_fname)
    after = pd.read_csv(after_msrs_fname)

    flt_before = before[~before["MSR_ADDR"].isin(MSR_EXCEPTION_LIST)]
    flt_after = after[~after["MSR_ADDR"].isin(MSR_EXCEPTION_LIST)]

    # Consider only values of MSRs which are present both before and after
    flt = (flt_before["STATUS"] == "implemented") & (
        flt_after["STATUS"] == "implemented"
    )
    impl_before = flt_before.loc[flt]
    impl_after = flt_after.loc[flt]

    status_diff = diff_msrs(before, after, column_to_drop="VALUE")
    value_diff = diff_msrs(impl_before, impl_after, column_to_drop="STATUS")

    assert_expr = not status_diff and not value_diff
    diag_output = (
        f"\n\n{guest_kernel_name} (status mismatch):\n"
        + status_diff
        + f"\n\n{guest_kernel_name} (value mismatch):\n"
        + value_diff
    )

    assert assert_expr, diag_output


def _test_cpu_wrmsr_restore(context):
    shared_names = context.custom["shared_names"]
    microvm_factory = context.custom["microvm_factory"]
    cpu_template = context.custom["cpu_template"]

    vm = microvm_factory.build()
    vm.spawn()

    iface = NetIfaceConfig()

    vm.create_tap_and_ssh_config(
        host_ip=iface.host_ip,
        guest_ip=iface.guest_ip,
        netmask_len=iface.netmask,
        tapname=iface.tap_name,
    )

    ssh_arti = context.disk.ssh_key()
    ssh_arti.download(vm.path)
    vm.ssh_config["ssh_key_path"] = ssh_arti.local_path()
    os.chmod(vm.ssh_config["ssh_key_path"], 0o400)

    cpu_template_dir = cpu_template if cpu_template else "none"
    snapshot_artifacts_dir = (
        Path(shared_names["snapshot_artifacts_root_dir_wrmsr"])
        / context.kernel.base_name()
        / cpu_template_dir
    )

    # Bring snapshot files from the 1st part of the test into the jail
    chroot_dir = vm.chroot()
    tmp_snapshot_artifacts_dir = (
        Path() / chroot_dir / "tmp" / context.kernel.base_name()
    )
    os.makedirs(tmp_snapshot_artifacts_dir)

    mem_fname_in_jail = Path(tmp_snapshot_artifacts_dir) / shared_names["mem_fname"]
    snapshot_fname_in_jail = (
        Path(tmp_snapshot_artifacts_dir) / shared_names["snapshot_fname"]
    )
    rootfs_fname_in_jail = (
        Path(tmp_snapshot_artifacts_dir) / shared_names["rootfs_fname"]
    )

    shutil.copyfile(
        Path(snapshot_artifacts_dir) / shared_names["mem_fname"],
        mem_fname_in_jail,
    )
    shutil.copyfile(
        Path(snapshot_artifacts_dir) / shared_names["snapshot_fname"],
        snapshot_fname_in_jail,
    )
    shutil.copyfile(
        Path(snapshot_artifacts_dir) / shared_names["rootfs_fname"],
        rootfs_fname_in_jail,
    )

    # Restore from the snapshot
    vm.restore_from_snapshot(
        snapshot_mem=mem_fname_in_jail,
        snapshot_vmstate=snapshot_fname_in_jail,
        snapshot_disks=[rootfs_fname_in_jail],
        snapshot_is_diff=True,
    )

    # Dump MSR state to a file for further comparison
    msrs_after_fname = Path(snapshot_artifacts_dir) / shared_names["msrs_after_fname"]
    ssh_connection = net_tools.SSHConnection(vm.ssh_config)

    dump_msr_state_to_file(msrs_after_fname, ssh_connection, shared_names)

    # Compare the two lists of MSR values and assert they are equal
    check_msr_values_are_equal(
        Path(snapshot_artifacts_dir) / shared_names["msrs_before_fname"],
        Path(snapshot_artifacts_dir) / shared_names["msrs_after_fname"],
        context.kernel.base_name(),  # this is to annotate the assertion output
    )


@pytest.mark.skipif(
    PLATFORM != "x86_64", reason="CPU features are masked only on x86_64."
)
@pytest.mark.skipif(
    cpuid_utils.get_cpu_vendor() != cpuid_utils.CpuVendor.INTEL,
    reason="CPU templates are only available on Intel.",
)
@pytest.mark.skipif(
    utils.get_kernel_version(level=1) not in SUPPORTED_KERNELS,
    reason=f"Supported kernels are {SUPPORTED_KERNELS}",
)
@pytest.mark.parametrize("cpu_template", SNAPSHOT_RESTORE_SHARED_NAMES["cpu_templates"])
@pytest.mark.timeout(900)
@pytest.mark.nonci
def test_cpu_wrmsr_restore(microvm_factory, cpu_template):
    """
    This is the second part of the test verifying
    that MSRs retain their values after restoring from a snapshot.

    Before taking a snapshot, MSR values are dumped into a text file.
    After restoring from the snapshot on another instance, the MSRs are
    dumped again and their values are compared to previous.
    Some MSRs are not inherently supposed to retain their values, so they
    form an MSR exception list.

    This part of the test is responsible for restoring from a snapshot and
    comparing two sets of MSR values.

    @type: functional
    """

    shared_names = SNAPSHOT_RESTORE_SHARED_NAMES

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    kernel_artifacts = ArtifactSet(artifacts.kernels())
    disk_artifacts = ArtifactSet(artifacts.disks(keyword=shared_names["disk_keyword"]))

    test_context = TestContext()
    test_context.custom = {
        "microvm_factory": microvm_factory,
        "cpu_template": cpu_template,
        "shared_names": shared_names,
    }

    test_matrix = TestMatrix(
        context=test_context,
        artifact_sets=[kernel_artifacts, disk_artifacts],
    )
    test_matrix.run_test(_test_cpu_wrmsr_restore)


@pytest.mark.skipif(
    PLATFORM != "x86_64", reason="CPU features are masked only on x86_64."
)
@pytest.mark.parametrize("cpu_template", ["T2", "T2S", "C3"])
def test_cpu_template(test_microvm_with_api, network_config, cpu_template):
    """
    Test masked and enabled cpu features against the expected template.

    This test checks that all expected masked features are not present in the
    guest and that expected enabled features are present for each of the
    supported CPU templates.

    @type: functional
    """
    test_microvm = test_microvm_with_api
    test_microvm.spawn()

    test_microvm.basic_config(vcpu_count=1)
    # Set template as specified in the `cpu_template` parameter.
    response = test_microvm.machine_cfg.put(
        vcpu_count=1,
        mem_size_mib=256,
        cpu_template=cpu_template,
    )
    assert test_microvm.api_session.is_status_no_content(response.status_code)
    _tap, _, _ = test_microvm.ssh_network_config(network_config, "1")

    response = test_microvm.actions.put(action_type="InstanceStart")
    if cpuid_utils.get_cpu_vendor() != cpuid_utils.CpuVendor.INTEL:
        # We shouldn't be able to apply Intel templates on AMD hosts
        assert test_microvm.api_session.is_status_bad_request(response.status_code)
        return

    assert test_microvm.api_session.is_status_no_content(response.status_code)
    check_masked_features(test_microvm, cpu_template)
    check_enabled_features(test_microvm, cpu_template)


def check_masked_features(test_microvm, cpu_template):
    """Verify the masked features of the given template."""
    common_masked_features_lscpu = [
        "dtes64",
        "monitor",
        "ds_cpl",
        "tm2",
        "cnxt-id",
        "sdbg",
        "xtpr",
        "pdcm",
        "osxsave",
        "psn",
        "ds",
        "acpi",
        "tm",
        "ss",
        "pbe",
        "fpdp",
        "rdt_m",
        "rdt_a",
        "mpx",
        "avx512f",
        "intel_pt",
        "avx512_vpopcntdq",
        "avx512_vnni",
        "3dnowprefetch",
        "pdpe1gb",
        "vmx",
        "umip",
    ]

    common_masked_features_cpuid = {
        "SGX": "false",
        "HLE": "false",
        "RTM": "false",
        "RDSEED": "false",
        "ADX": "false",
        "AVX512IFMA": "false",
        "CLFLUSHOPT": "false",
        "CLWB": "false",
        "AVX512PF": "false",
        "AVX512ER": "false",
        "AVX512CD": "false",
        "SHA": "false",
        "AVX512BW": "false",
        "AVX512VL": "false",
        "AVX512VBMI": "false",
        "PKU": "false",
        "OSPKE": "false",
        "RDPID": "false",
        "SGX_LC": "false",
        "AVX512_4VNNIW": "false",
        "AVX512_4FMAPS": "false",
        "XSAVEC": "false",
        "XGETBV": "false",
        "XSAVES": "false",
        "UMIP": "false",
        "VMX": "false",
    }

    # These are all discoverable by cpuid -1.
    c3_masked_features = {
        "FMA": "false",
        "MOVBE": "false",
        "BMI": "false",
        "AVX2": "false",
        "BMI2": "false",
        "INVPCID": "false",
    }

    # Check that all common features discoverable with lscpu
    # are properly masked.
    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)
    guest_cmd = "cat /proc/cpuinfo | grep 'flags' | head -1"
    _, stdout, stderr = ssh_connection.execute_command(guest_cmd)
    assert stderr.read() == ""

    cpu_flags_output = stdout.readline().rstrip().split(" ")

    for feature in common_masked_features_lscpu:
        assert feature not in cpu_flags_output, feature

    # Check that all common features discoverable with cpuid
    # are properly masked.
    cpuid_utils.check_guest_cpuid_output(
        test_microvm, "cpuid -1", None, "=", common_masked_features_cpuid
    )

    if cpu_template == "C3":
        cpuid_utils.check_guest_cpuid_output(
            test_microvm, "cpuid -1", None, "=", c3_masked_features
        )

    # Check if XSAVE PKRU is masked for T3/C2.
    expected_cpu_features = {"XCR0 supported: PKRU state": "false"}

    cpuid_utils.check_guest_cpuid_output(
        test_microvm, "cpuid -1", None, "=", expected_cpu_features
    )


def check_enabled_features(test_microvm, cpu_template):
    """Test for checking that all expected features are enabled in guest."""
    enabled_list = {  # feature_info_1_edx
        "x87 FPU on chip": "true",
        "CMPXCHG8B inst": "true",
        "virtual-8086 mode enhancement": "true",
        "SSE extensions": "true",
        "SSE2 extensions": "true",
        "debugging extensions": "true",
        "page size extensions": "true",
        "time stamp counter": "true",
        "RDMSR and WRMSR support": "true",
        "physical address extensions": "true",
        "machine check exception": "true",
        "APIC on chip": "true",
        "MMX Technology": "true",
        "SYSENTER and SYSEXIT": "true",
        "memory type range registers": "true",
        "PTE global bit": "true",
        "FXSAVE/FXRSTOR": "true",
        "machine check architecture": "true",
        "conditional move/compare instruction": "true",
        "page attribute table": "true",
        "page size extension": "true",
        "CLFLUSH instruction": "true",
        # feature_info_1_ecx
        "PNI/SSE3: Prescott New Instructions": "true",
        "PCLMULDQ instruction": "true",
        "SSSE3 extensions": "true",
        "AES instruction": "true",
        "CMPXCHG16B instruction": "true",
        "process context identifiers": "true",
        "SSE4.1 extensions": "true",
        "SSE4.2 extensions": "true",
        "extended xAPIC support": "true",
        "POPCNT instruction": "true",
        "time stamp counter deadline": "true",
        "XSAVE/XSTOR states": "true",
        "OS-enabled XSAVE/XSTOR": "true",
        "AVX: advanced vector extensions": "true",
        "F16C half-precision convert instruction": "true",
        "RDRAND instruction": "true",
        "hypervisor guest status": "true",
        # thermal_and_power_mgmt
        "ARAT always running APIC timer": "true",
        # extended_features
        "FSGSBASE instructions": "true",
        "IA32_TSC_ADJUST MSR supported": "true",
        "SMEP supervisor mode exec protection": "true",
        "enhanced REP MOVSB/STOSB": "true",
        "SMAP: supervisor mode access prevention": "true",
        # xsave_0xd_0
        "XCR0 supported: x87 state": "true",
        "XCR0 supported: SSE state": "true",
        "XCR0 supported: AVX state": "true",
        # xsave_0xd_1
        "XSAVEOPT instruction": "true",
        # extended_080000001_edx
        "SYSCALL and SYSRET instructions": "true",
        "64-bit extensions technology available": "true",
        "execution disable": "true",
        "RDTSCP": "true",
        # intel_080000001_ecx
        "LAHF/SAHF supported in 64-bit mode": "true",
        # adv_pwr_mgmt
        "TscInvariant": "true",
    }

    cpuid_utils.check_guest_cpuid_output(
        test_microvm, "cpuid -1", None, "=", enabled_list
    )
    if cpu_template == "T2":
        t2_enabled_features = {
            "FMA": "true",
            "BMI": "true",
            "BMI2": "true",
            "AVX2": "true",
            "MOVBE": "true",
            "INVPCID": "true",
        }
        cpuid_utils.check_guest_cpuid_output(
            test_microvm, "cpuid -1", None, "=", t2_enabled_features
        )
