# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

"""Tests for the verifying features exposed by CPUID and MSRs by various CPU templates."""

import pytest

from conftest import _test_images_s3_bucket
from framework.artifacts import ArtifactCollection, ArtifactSet
from framework.matrix import TestMatrix, TestContext
from framework.builder import MicrovmBuilder
import framework.utils_cpuid as cpuid_utils
import framework.utils_cpu_templates as cputmpl_utils
import host_tools.network as net_tools


# CPU templates designed to provide instruction set feature parity
INST_SET_TEMPLATES = ["T2A", "T2CL"]


def get_guest_kernel_ver(vm):
    """
    Returns the guest kernel version.
    Useful when running test matrix with multiple guest kernels.
    """
    ssh_conn = net_tools.SSHConnection(vm.ssh_config)
    read_kernel_ver_cmd = "uname -r"
    _, stdout, stderr = ssh_conn.execute_command(read_kernel_ver_cmd)
    assert stderr.read() == ""
    return stdout.read().strip()


def _test_cpuid_feat_flags(context):
    vm_builder = context.custom["builder"]
    root_disk = context.disk.copy()
    cpu_template = context.custom["cpu_template"]
    must_be_set = context.custom["flags_must_be_set"]
    must_be_unset = context.custom["flags_must_be_unset"]

    vm_instance = vm_builder.build(
        kernel=context.kernel,
        disks=[root_disk],
        ssh_key=context.disk.ssh_key(),
        config=context.microvm,
        cpu_template=cpu_template,
    )
    vm = vm_instance.vm
    vm.start()

    cpuid = cpuid_utils.get_guest_cpuid(vm)
    kernel_ver = get_guest_kernel_ver(vm)
    allowed_regs = ["eax", "ebx", "ecx", "edx"]

    for leaf, subleaf, reg, flags in must_be_set:
        assert reg in allowed_regs
        actual = cpuid[(leaf, subleaf, reg)] & flags
        expected = flags
        assert (
            actual == expected
        ), f"{cpu_template}: {kernel_ver=} {leaf=:#x} {subleaf=:#x} {reg=} {actual=:#x}, {expected=:#x}"

    for leaf, subleaf, reg, flags in must_be_unset:
        assert reg in allowed_regs
        actual = cpuid[(leaf, subleaf, reg)] & flags
        expected = 0
        assert (
            actual == expected
        ), f"{cpu_template} {kernel_ver=} {leaf=:#x} {subleaf=:#x} {reg=} {actual=:#x}, {expected=:#x}"


def _test_cpuid_feat_flags_matrix(
    bin_cloner_path,
    network_config,
    cpu_template,
    flags_must_be_set,
    flags_must_be_unset,
):
    """
    This launches tests matrix for CPUID feature flag checks for the given CPU template.
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
        "flags_must_be_set": flags_must_be_set,
        "flags_must_be_unset": flags_must_be_unset,
    }
    test_matrix = TestMatrix(
        context=test_context,
        artifact_sets=[microvm_artifacts, kernel_artifacts, disk_artifacts],
    )
    test_matrix.run_test(_test_cpuid_feat_flags)


@pytest.mark.parametrize(
    "cpu_template",
    cputmpl_utils.select_supported_cpu_templates(cputmpl_utils.ALL_TEMPLATES),
)
def test_feat_parity_cpuid_mpx(bin_cloner_path, network_config, cpu_template):
    """
    Verifies that MPX (Memory Protection Extensions) is not enabled in any of the supported CPU templates.

    @type: functional
    """
    # fmt: off
    must_be_set = []
    must_be_unset = [
        (0x7, 0x0, "ebx",
            (1 << 14) # MPX
        ),
    ]
    # fmt: on

    _test_cpuid_feat_flags_matrix(
        bin_cloner_path, network_config, cpu_template, must_be_set, must_be_unset
    )


@pytest.mark.parametrize(
    "cpu_template",
    cputmpl_utils.select_supported_cpu_templates(INST_SET_TEMPLATES + ["T2"]),
)
def test_feat_parity_cpuid_inst_set(bin_cloner_path, network_config, cpu_template):
    """
    Verifies that CPUID feature flags related to instruction sets are properly set
    for T2, T2CL and T2A CPU templates.

    @type: functional
    """

    # fmt: off
    must_be_set = [
        (0x7, 0x0, "ebx",
            (1 << 5) | # AVX2
            (1 << 9) # REP MOVSB/STOSB
        ),
    ]

    must_be_unset = [
        # Instruction set related
        (0x1, 0x0, "ecx",
            (1 << 15) # PDCM
        ),
        (0x7, 0x0, "ebx",
            (1 << 16) | # AVX512F
            (1 << 17) | # AVX512DQ
            (1 << 18) | # RDSEED
            (1 << 19) | # ADX
            (1 << 23) | # CLFLUSHOPT
            (1 << 24) | # CLWB
            (1 << 29) | # SHA
            (1 << 30) | # AVX512BW
            (1 << 31) # AVX512VL
        ),
        (0x7, 0x0, "ecx",
            (1 << 1) | # AVX512_VBMI
            (1 << 6) | # AVX512_VBMI2
            (1 << 8) | # GFNI
            (1 << 9) | # VAES
            (1 << 10) | # VPCLMULQDQ
            (1 << 11) | # AVX512_VNNI
            (1 << 12) | # AVX512_BITALG
            (1 << 14) | # AVX512_VPOPCNTDQ
            (1 << 22) # RDPID/IA32_TSC_AUX
        ),
        (0x7, 0x0, "edx",
            (1 << 2) | # AVX512_4VNNIW
            (1 << 3) | # AVX512_4FMAPS
            (1 << 4) | # Fast Short REP MOV
            (1 << 8) # AVX512_VP2INTERSECT
        ),
        (0x80000001, 0x0, "ecx",
            (1 << 6) | # SSE4A
            (1 << 7) | # MisAlignSee
            (1 << 8) | # PREFETCHW
            (1 << 29) # MwaitExtended
        ),
        (0x80000001, 0x0, "edx",
            (1 << 22) | # MmxExt
            (1 << 23) | # MMX
            (1 << 24) | # FXSR
            (1 << 25) # FFXSR
        ),
        (0x80000008, 0x0, "ebx",
            (1 << 0) | # CLZERO
            (1 << 2) | # RstrFpErrPtrs
            (1 << 4) | # RDPRU
            (1 << 8) | # MCOMMIT
            (1 << 9) | # WBNOINVD
            (1 << 13) # INT_WBINVD
        ),
    ]
    # fmt: on

    _test_cpuid_feat_flags_matrix(
        bin_cloner_path, network_config, cpu_template, must_be_set, must_be_unset
    )


@pytest.mark.parametrize(
    "cpu_template", cputmpl_utils.select_supported_cpu_templates(INST_SET_TEMPLATES)
)
def test_feat_parity_cpuid_sec(bin_cloner_path, network_config, cpu_template):
    """
    Verifies that security-related CPUID feature flags are properly set
    for T2CL and T2A CPU templates.

    @type: functional
    """

    # fmt: off
    must_be_set_common = [
        # Security related
        (0x7, 0x0, "edx",
            (1 << 26) | # IBRS/IBPB
            (1 << 27) | # STIBP
            (1 << 31) # SSBD
        )
        # Security feature bits in 0x80000008 EBX are set differently by
        # 4.14 and 5.10 KVMs.
        # 4.14 populates them from host's AMD flags (0x80000008 EBX), while
        # 5.10 takes them from host's common flags (0x7 EDX).
        # There is no great value in checking that this actually happens, as
        # we cannot really control it.
        # When we drop 4.14 support, we may consider enabling this check.
        # (0x80000008, 0x0, "ebx",
        #     (1 << 12) | # IBPB
        #     (1 << 14) | # IBRS
        #     (1 << 15) | # STIBP
        #     (1 << 24) # SSBD
        # )
    ]

    must_be_set_intel_only = [
        # Security related
        (0x7, 0x0, "edx",
            (1 << 10) | # MD_CLEAR
            (1 << 29) # IA32_ARCH_CAPABILITIES
        )
    ]

    must_be_set_amd_only = [
        # Security related
        (0x80000008, 0x0, "ebx",
            (1 << 18) | # IbrsPreferred
            (1 << 19) # IbrsProvidesSameModeProtection
        )
    ]

    must_be_unset_common = [
        # Security related
        (0x7, 0x0, "edx",
            (1 << 28) # L1D_FLUSH
        )
    ]

    must_be_unset_intel_only = [
        # Security related
        (0x80000008, 0x0, "ebx",
            (1 << 18) | # IbrsPreferred
            (1 << 19) # IbrsProvidesSameModeProtection
        )
    ]

    must_be_unset_amd_only = [
        # Security related
        (0x7, 0x0, "edx",
            (1 << 10) | # MD_CLEAR
            (1 << 29) # IA32_ARCH_CAPABILITIES
        )
    ]
    # fmt: on

    vendor = cpuid_utils.get_cpu_vendor()
    if vendor == cpuid_utils.CpuVendor.INTEL:
        must_be_set = must_be_set_common + must_be_set_intel_only
        must_be_unset = must_be_unset_common + must_be_unset_intel_only
    elif vendor == cpuid_utils.CpuVendor.AMD:
        must_be_set = must_be_set_common + must_be_set_amd_only
        must_be_unset = must_be_unset_common + must_be_unset_amd_only

    _test_cpuid_feat_flags_matrix(
        bin_cloner_path, network_config, cpu_template, must_be_set, must_be_unset
    )


def check_arch_cap_msr(vm, cpu_template):
    """
    Checks that IA32_ARCH_CAPABILITIES MSR has the expected value.
    """
    kernel_ver = get_guest_kernel_ver(vm)

    ssh_conn = net_tools.SSHConnection(vm.ssh_config)
    rdmsr_cmd = "rdmsr 0x10a"
    _, stdout, stderr = ssh_conn.execute_command(rdmsr_cmd)

    if cpu_template == "T2CL":
        assert stderr.read() == "", f"{kernel_ver=}"
        actual = int(stdout.read().strip(), 16)
        expected = 0xEB
        assert (
            actual == expected
        ), f"{cpu_template}: {kernel_ver=}, {actual=:#x}, {expected=:#x}"
    elif cpu_template == "T2A":
        # IA32_ARCH_CAPABILITIES shall not be available
        assert stderr.read() != "", f"{cpu_template}: {kernel_ver=}"


def _test_msr_arch_cap(context):
    vm_builder = context.custom["builder"]
    root_disk = context.disk.copy()
    cpu_template = context.custom["cpu_template"]

    vm_instance = vm_builder.build(
        kernel=context.kernel,
        disks=[root_disk],
        ssh_key=context.disk.ssh_key(),
        config=context.microvm,
        cpu_template=cpu_template,
    )
    vm = vm_instance.vm
    vm.start()

    check_arch_cap_msr(vm, cpu_template)


@pytest.mark.parametrize(
    "cpu_template", cputmpl_utils.select_supported_cpu_templates(INST_SET_TEMPLATES)
)
def test_feat_parity_msr_arch_cap(bin_cloner_path, network_config, cpu_template):
    """
    Verifies availability and value of the IA32_ARCH_CAPABILITIES MSR for T2CL and T2A CPU templates.

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
    test_matrix.run_test(_test_msr_arch_cap)
