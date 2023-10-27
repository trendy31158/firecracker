# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
#
# N.B.: Although this repository is released under the Apache-2.0, part of its test requires a
# script from the third party "Spectre & Meltdown Checker" project. This script is under the
# GPL-3.0-only license.
"""Tests vulnerabilities mitigations."""
import json
import os

import pytest
import requests

from framework.ab_test import (
    git_ab_test_guest_command,
    git_ab_test_guest_command_if_pr,
    git_ab_test_host_command_if_pr,
    is_pr,
    set_did_not_grow_comparator,
)
from framework.properties import global_props
from framework.utils import CommandReturn
from framework.utils_cpu_templates import nonci_on_arm

CHECKER_URL = "https://meltdown.ovh"
CHECKER_FILENAME = "spectre-meltdown-checker.sh"
REMOTE_CHECKER_PATH = f"/tmp/{CHECKER_FILENAME}"
REMOTE_CHECKER_COMMAND = f"sh {REMOTE_CHECKER_PATH} --no-intel-db --batch json"

VULN_DIR = "/sys/devices/system/cpu/vulnerabilities"


def configure_microvm(
    factory,
    kernel,
    rootfs,
    *,
    firecracker=None,
    jailer=None,
    cpu_template=None,
    custom_cpu_template=None,
):
    """Build a microvm for vulnerability tests"""
    assert not (cpu_template and custom_cpu_template)
    # Either both or neither are specified
    assert firecracker and jailer or not firecracker and not jailer

    if firecracker:
        microvm = factory.build(
            kernel, rootfs, fc_binary_path=firecracker, jailer_binary_path=jailer
        )
    else:
        microvm = factory.build(kernel, rootfs)

    microvm.spawn()
    microvm.basic_config(vcpu_count=2, mem_size_mib=256, cpu_template=cpu_template)
    if custom_cpu_template:
        microvm.api.cpu_config.put(**custom_cpu_template["template"])
    microvm.cpu_template = cpu_template
    if cpu_template is None and custom_cpu_template is not None:
        microvm.cpu_template = custom_cpu_template["name"]
    microvm.add_net_iface()
    microvm.start()
    return microvm


@pytest.fixture
def build_microvm(
    microvm_factory,
    guest_kernel_linux_5_10,
    rootfs_ubuntu_22,
):
    """Fixture returning a factory function for a normal microvm"""
    return lambda firecracker=None, jailer=None: configure_microvm(
        microvm_factory,
        guest_kernel_linux_5_10,
        rootfs_ubuntu_22,
        firecracker=firecracker,
        jailer=jailer,
    )


@pytest.fixture
def build_microvm_with_template(
    microvm_factory, guest_kernel_linux_5_10, rootfs_ubuntu_22, cpu_template
):
    """Fixture returning a factory function for microvms with our built-in template"""
    return lambda firecracker=None, jailer=None: configure_microvm(
        microvm_factory,
        guest_kernel_linux_5_10,
        rootfs_ubuntu_22,
        firecracker=firecracker,
        jailer=jailer,
        cpu_template=cpu_template,
    )


@pytest.fixture
def build_microvm_with_custom_template(
    microvm_factory, guest_kernel_linux_5_10, rootfs_ubuntu_22, custom_cpu_template
):
    """Fixture returning a factory function for microvms with custom cpu templates"""
    return lambda firecracker=None, jailer=None: configure_microvm(
        microvm_factory,
        guest_kernel_linux_5_10,
        rootfs_ubuntu_22,
        firecracker=firecracker,
        jailer=jailer,
        custom_cpu_template=custom_cpu_template,
    )


def with_restore(factory, microvm_factory):
    """Turns the given microvm factory into one that makes the microvm go through a snapshot-restore cycle"""

    def restore(firecracker=None, jailer=None):
        microvm = factory(firecracker, jailer)
        snapshot = microvm.snapshot_full()

        if firecracker:
            dst_vm = microvm_factory.build(
                fc_binary_path=firecracker, jailer_binary_path=jailer
            )
        else:
            dst_vm = microvm_factory.build()
        dst_vm.spawn()
        # Restore the destination VM from the snapshot
        dst_vm.restore_from_snapshot(snapshot, resume=True)
        dst_vm.cpu_template = microvm.cpu_template

        return dst_vm

    return restore


def with_checker(factory, spectre_meltdown_checker):
    """Turns the given microvm factory function into one that also contains the spectre-meltdown checker script"""

    def download_checker(firecracker, jailer):
        microvm = factory(firecracker, jailer)
        microvm.ssh.scp_put(spectre_meltdown_checker, REMOTE_CHECKER_PATH)
        return microvm

    return download_checker


@pytest.fixture(scope="session", name="spectre_meltdown_checker")
def download_spectre_meltdown_checker(tmp_path_factory):
    """Download spectre / meltdown checker script."""
    resp = requests.get(CHECKER_URL, timeout=5)
    resp.raise_for_status()

    path = tmp_path_factory.mktemp("tmp", True) / CHECKER_FILENAME
    path.write_bytes(resp.content)

    return path


def spectre_meltdown_reported_vulnerablities(
    spectre_meltdown_checker_output: CommandReturn,
) -> set:
    """Parses the output of `spectre-meltdown-checker.sh --batch json` and returns the set of issues for which it reported 'Vulnerable'"""
    return {
        frozenset(entry)  # cannot hash dicts
        for entry in json.loads(spectre_meltdown_checker_output.stdout)
        if entry["VULNERABLE"]
    }


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
def test_spectre_meltdown_checker_on_host(spectre_meltdown_checker):
    """
    Test with the spectre / meltdown checker on host.
    """
    git_ab_test_host_command_if_pr(
        f"sh {spectre_meltdown_checker} --batch json",
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
def test_spectre_meltdown_checker_on_guest(spectre_meltdown_checker, build_microvm):
    """
    Test with the spectre / meltdown checker on guest.
    """

    git_ab_test_guest_command_if_pr(
        with_checker(build_microvm, spectre_meltdown_checker),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
def test_spectre_meltdown_checker_on_restored_guest(
    spectre_meltdown_checker, build_microvm, microvm_factory
):
    """
    Test with the spectre / meltdown checker on a restored guest.
    """
    git_ab_test_guest_command_if_pr(
        with_checker(
            with_restore(build_microvm, microvm_factory), spectre_meltdown_checker
        ),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
@nonci_on_arm
def test_spectre_meltdown_checker_on_guest_with_template(
    spectre_meltdown_checker, build_microvm_with_template
):
    """
    Test with the spectre / meltdown checker on guest with CPU template.
    """

    git_ab_test_guest_command_if_pr(
        with_checker(build_microvm_with_template, spectre_meltdown_checker),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
@nonci_on_arm
def test_spectre_meltdown_checker_on_guest_with_custom_template(
    spectre_meltdown_checker, build_microvm_with_custom_template
):
    """
    Test with the spectre / meltdown checker on guest with a custom CPU template.
    """
    git_ab_test_guest_command_if_pr(
        with_checker(build_microvm_with_custom_template, spectre_meltdown_checker),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
@nonci_on_arm
def test_spectre_meltdown_checker_on_restored_guest_with_template(
    spectre_meltdown_checker, build_microvm_with_template, microvm_factory
):
    """
    Test with the spectre / meltdown checker on a restored guest with a CPU template.
    """
    git_ab_test_guest_command_if_pr(
        with_checker(
            with_restore(build_microvm_with_template, microvm_factory),
            spectre_meltdown_checker,
        ),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


@pytest.mark.skipif(
    global_props.instance == "c7g.metal" and global_props.host_linux_version == "4.14",
    reason="c7g host 4.14 requires modifications to the 5.10 guest kernel to boot successfully.",
)
@nonci_on_arm
def test_spectre_meltdown_checker_on_restored_guest_with_custom_template(
    spectre_meltdown_checker, build_microvm_with_custom_template, microvm_factory
):
    """
    Test with the spectre / meltdown checker on a restored guest with a custom CPU template.
    """
    git_ab_test_guest_command_if_pr(
        with_checker(
            with_restore(build_microvm_with_custom_template, microvm_factory),
            spectre_meltdown_checker,
        ),
        REMOTE_CHECKER_COMMAND,
        comparator=set_did_not_grow_comparator(
            spectre_meltdown_reported_vulnerablities
        ),
    )


def get_vuln_files_exception_dict(template):
    """
    Returns a dictionary of expected values for vulnerability files requiring special treatment.
    """
    exception_dict = {}

    # Exception for mmio_stale_data
    # =============================
    #
    # Guests on Intel Skylake or with T2S template
    # --------------------------------------------
    # Whether mmio_stale_data is marked as "Vulnerable" or not is determined by the code here.
    # https://elixir.bootlin.com/linux/v6.1.46/source/arch/x86/kernel/cpu/bugs.c#L431
    # Virtualization of FLUSH_L1D has been available and CPUID.(EAX=0x7,ECX=0):EDX[28 (FLUSH_L1D)]
    # has been passed through to guests only since kernel v6.4.
    # https://github.com/torvalds/linux/commit/da3db168fb671f15e393b227f5c312c698ecb6ea
    # Thus, since the FLUSH_L1D bit is masked off prior to kernel v6.4, guests with
    # IA32_ARCH_CAPABILITIES.FB_CLEAR (bit 17) = 0 (like guests on Intel Skylake and guests with
    # T2S template) fall onto the second hand of the condition and fail the test. The expected value
    # "Vulnerable: Clear CPU buffers attempted, no microcode" means that the kernel is using the
    # best effort mode which invokes the mitigation instructions (VERW in this case) without a
    # guarantee that they clear the CPU buffers. If the host has the microcode update applied
    # correctly, the mitigation works and it is safe to ignore the "Vulnerable" message.
    #
    # Guest on Intel Skylake with C3 template
    # ---------------------------------------
    # If the processor does not enumerate IA32_ARCH_CAPABILITIES.{FBSDP_NO,PSDP_NO,SBDR_SSDP_NO},
    # the kernel checks its lists of affected/unaffected processors and determines whether the
    # mitigation is required, and if the processor is not included in the lists, the sysfs is marked
    # as "Unknown".
    # https://elixir.bootlin.com/linux/v6.1.50/source/arch/x86/kernel/cpu/common.c#L1387
    # The behavior for "Unknown" state was added in the following commit and older processors that
    # are no longer serviced are not listed up.
    # https://github.com/torvalds/linux/commit/7df548840c496b0141fb2404b889c346380c2b22
    # Since those bits are not set on Intel Skylake and C3 template makes guests pretend to be AWS
    # C3 instance (quite old processor now) by overwriting CPUID.1H:EAX, it is impossible to avoid
    # this "Unknown" state.
    if global_props.cpu_codename == "INTEL_SKYLAKE" and template == "C3":
        exception_dict["mmio_stale_data"] = "Unknown: No mitigations"
    elif global_props.cpu_codename == "INTEL_SKYLAKE" or template == "T2S":
        exception_dict[
            "mmio_stale_data"
        ] = "Vulnerable: Clear CPU buffers attempted, no microcode"

    return exception_dict


def test_vulnerabilities_on_host():
    """
    Test vulnerabilities files on host.
    """

    git_ab_test_host_command_if_pr(
        f"! grep -r Vulnerable {VULN_DIR}",
        comparator=set_did_not_grow_comparator(
            lambda output: set(output.stdout.splitlines())
        ),
    )


def check_vulnerabilities_files_on_guest(microvm):
    """
    Check that the guest's vulnerabilities files do not contain `Vulnerable`.
    See also: https://elixir.bootlin.com/linux/latest/source/Documentation/ABI/testing/sysfs-devices-system-cpu
    and search for `vulnerabilities`.
    """
    # Retrieve a list of vulnerabilities files available inside guests.
    vuln_dir = "/sys/devices/system/cpu/vulnerabilities"
    ecode, stdout, stderr = microvm.ssh.run(f"find {vuln_dir} -type f")
    assert ecode == 0, f"stdout:\n{stdout}\nstderr:\n{stderr}\n"
    vuln_files = stdout.split("\n")

    # Fixtures in this file (test_vulnerabilities.py) add this special field.
    template = microvm.cpu_template

    # Check that vulnerabilities files in the exception dictionary have the expected values and
    # the others do not contain "Vulnerable".
    exceptions = get_vuln_files_exception_dict(template)
    for vuln_file in vuln_files:
        filename = os.path.basename(vuln_file)
        if filename in exceptions:
            _, stdout, _ = microvm.ssh.run(f"cat {vuln_file}")
            assert exceptions[filename] in stdout
        else:
            cmd = f"grep Vulnerable {vuln_file}"
            ecode, stdout, stderr = microvm.ssh.run(cmd)
            assert ecode == 1, f"{vuln_file}: stdout:\n{stdout}\nstderr:\n{stderr}\n"


def check_vulnerabilities_files_ab(builder):
    """Does an A/B test on the contents of the /sys/devices/system/cpu/vulnerabilities files in the guest if
    running in a PR pipeline, and otherwise calls `check_vulnerabilities_files_on_guest`
    """
    if is_pr():
        git_ab_test_guest_command(
            builder,
            f"! grep -r Vulnerable {VULN_DIR}",
            comparator=set_did_not_grow_comparator(
                lambda output: set(output.stdout.splitlines())
            ),
        )
    else:
        check_vulnerabilities_files_on_guest(builder())


def test_vulnerabilities_files_on_guest(build_microvm):
    """
    Test vulnerabilities files on guest.
    """
    check_vulnerabilities_files_ab(build_microvm)


def test_vulnerabilities_files_on_restored_guest(build_microvm, microvm_factory):
    """
    Test vulnerabilities files on a restored guest.
    """
    check_vulnerabilities_files_ab(with_restore(build_microvm, microvm_factory))


@nonci_on_arm
def test_vulnerabilities_files_on_guest_with_template(build_microvm_with_template):
    """
    Test vulnerabilities files on guest with CPU template.
    """
    check_vulnerabilities_files_ab(build_microvm_with_template)


@nonci_on_arm
def test_vulnerabilities_files_on_guest_with_custom_template(
    build_microvm_with_custom_template,
):
    """
    Test vulnerabilities files on guest with a custom CPU template.
    """
    check_vulnerabilities_files_ab(build_microvm_with_custom_template)


@nonci_on_arm
def test_vulnerabilities_files_on_restored_guest_with_template(
    build_microvm_with_template, microvm_factory
):
    """
    Test vulnerabilities files on a restored guest with a CPU template.
    """
    check_vulnerabilities_files_ab(
        with_restore(build_microvm_with_template, microvm_factory)
    )


@nonci_on_arm
def test_vulnerabilities_files_on_restored_guest_with_custom_template(
    build_microvm_with_custom_template, microvm_factory
):
    """
    Test vulnerabilities files on a restored guest with a custom CPU template.
    """
    check_vulnerabilities_files_ab(
        with_restore(build_microvm_with_custom_template, microvm_factory)
    )
