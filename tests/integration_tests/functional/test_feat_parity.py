# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

"""Tests for the verifying features exposed by CPUID and MSRs by various CPU templates."""

import pytest

from conftest import ARTIFACTS_COLLECTION
from framework.artifacts import DiskArtifact
from framework.builder import MicrovmBuilder
import framework.utils_cpuid as cpuid_utils
from framework.utils_cpu_templates import SUPPORTED_CPU_TEMPLATES


# CPU templates designed to provide instruction set feature parity
INST_SET_TEMPLATES = ["T2A", "T2CL"]


@pytest.fixture(name="vm_builder", scope="session")
def vm_builder_fxt(bin_cloner_path):
    """Return a microvm builder."""
    return MicrovmBuilder(bin_cloner_path)


@pytest.fixture(
    name="microvm",
    params=ARTIFACTS_COLLECTION.microvms(keyword="1vcpu_1024mb"),
    ids=lambda uvm: uvm.name(),
)
def microvm_fxt(request):
    """Common microvm fixture for tests in this file"""
    uvm = request.param
    uvm.download()
    return uvm


@pytest.fixture(
    name="disk",
    params=ARTIFACTS_COLLECTION.disks(keyword="bionic-msrtools"),
    ids=lambda disk: disk.name() if isinstance(disk, DiskArtifact) else None,
)
def disk_fxt(request):
    """Common disk fixture for tests in this file"""
    disk = request.param
    disk.download()
    return disk


@pytest.fixture(
    name="inst_set_cpu_template",
    params=set(SUPPORTED_CPU_TEMPLATES).intersection(INST_SET_TEMPLATES),
)
def inst_set_cpu_template_fxt(request):
    """CPU template fixture for instruction set feature parity templates"""
    return request.param


@pytest.fixture(
    name="inst_set_cpu_template_ext",
    params=set(SUPPORTED_CPU_TEMPLATES).intersection(INST_SET_TEMPLATES + ["T2"]),
)
def inst_set_cpu_template_ext_fxt(request):
    """CPU template fixture for instruction set feature parity templates plus T2"""
    return request.param


def create_vm(vm_builder, cpu_template, microvm, kernel, disk):
    """
    Create a VM.
    """
    root_disk = disk.copy()
    vm_instance = vm_builder.build(
        kernel=kernel,
        disks=[root_disk],
        ssh_key=disk.ssh_key(),
        config=microvm,
        cpu_template=cpu_template,
    )
    vm = vm_instance.vm

    return vm


def check_cpuid_feat_flags(
    vm_builder, cpu_template, microvm, kernel, disk, must_be_set, must_be_unset
):
    """
    Check that CPUID feature flag are set and unset as expected.
    """
    vm = create_vm(vm_builder, cpu_template, microvm, kernel, disk)
    vm.start()

    cpuid = cpuid_utils.get_guest_cpuid(vm)
    allowed_regs = ["eax", "ebx", "ecx", "edx"]

    for leaf, subleaf, reg, flags in must_be_set:
        assert reg in allowed_regs
        actual = cpuid[(leaf, subleaf, reg)] & flags
        expected = flags
        assert (
            actual == expected
        ), f"{leaf=:#x} {subleaf=:#x} {reg=} {actual=:#x}, {expected=:#x}"

    for leaf, subleaf, reg, flags in must_be_unset:
        assert reg in allowed_regs
        actual = cpuid[(leaf, subleaf, reg)] & flags
        expected = 0
        assert (
            actual == expected
        ), f"{leaf=:#x} {subleaf=:#x} {reg=} {actual=:#x}, {expected=:#x}"


def test_feat_parity_cpuid_mpx(vm_builder, cpu_template, microvm, guest_kernel, disk):
    """
    Verify that MPX (Memory Protection Extensions) is not enabled in any of the supported CPU templates.

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

    check_cpuid_feat_flags(
        vm_builder,
        cpu_template,
        microvm,
        guest_kernel,
        disk,
        must_be_set,
        must_be_unset,
    )


def test_feat_parity_cpuid_inst_set(
    vm_builder, inst_set_cpu_template_ext, microvm, guest_kernel, disk
):
    """
    Verify that CPUID feature flags related to instruction sets are properly set
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

    check_cpuid_feat_flags(
        vm_builder,
        inst_set_cpu_template_ext,
        microvm,
        guest_kernel,
        disk,
        must_be_set,
        must_be_unset,
    )


def test_feat_parity_cpuid_sec(
    vm_builder, inst_set_cpu_template, microvm, guest_kernel, disk
):
    """
    Verify that security-related CPUID feature flags are properly set
    for T2CL and T2A CPU templates.

    @type: functional
    """

    # fmt: off
    must_be_set_common = [
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
        (0x7, 0x0, "edx",
            (1 << 10) | # MD_CLEAR
            (1 << 29) # IA32_ARCH_CAPABILITIES
        )
    ]

    must_be_set_amd_only = [
        (0x80000008, 0x0, "ebx",
            (1 << 18) | # IbrsPreferred
            (1 << 19) # IbrsProvidesSameModeProtection
        )
    ]

    must_be_unset_common = [
        (0x7, 0x0, "edx",
            (1 << 28) # L1D_FLUSH
        )
    ]

    must_be_unset_intel_only = [
        (0x80000008, 0x0, "ebx",
            (1 << 18) | # IbrsPreferred
            (1 << 19) # IbrsProvidesSameModeProtection
        )
    ]

    must_be_unset_amd_only = [
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
    else:
        raise Exception("Unsupported CPU vendor.")

    check_cpuid_feat_flags(
        vm_builder,
        inst_set_cpu_template,
        microvm,
        guest_kernel,
        disk,
        must_be_set,
        must_be_unset,
    )


def test_feat_parity_msr_arch_cap(
    vm_builder, inst_set_cpu_template, microvm, guest_kernel, disk
):
    """
    Verify availability and value of the IA32_ARCH_CAPABILITIES MSR for T2CL and T2A CPU templates.

    @type: functional
    """
    vm = create_vm(vm_builder, inst_set_cpu_template, microvm, guest_kernel, disk)
    vm.start()

    arch_capabilities_addr = "0x10a"
    rdmsr_cmd = f"rdmsr {arch_capabilities_addr}"
    _, stdout, stderr = vm.ssh.execute_command(rdmsr_cmd)

    if inst_set_cpu_template == "T2CL":
        assert stderr.read() == ""
        actual = int(stdout.read().strip(), 16)
        # fmt: off
        expected = (
            (1 << 0) | # RDCL_NO
            (1 << 1) | # IBRS_ALL
            (1 << 3) | # SKIP_L1DFL_VMENTRY
            (1 << 5) | # MDS_NO
            (1 << 6) | # IF_PSCHANGE_MC_NO
            (1 << 7) # TSX_CTRL
        )
        # fmt: on
        assert actual == expected, f"{actual=:#x} != {expected=:#x}"
    elif inst_set_cpu_template == "T2A":
        # IA32_ARCH_CAPABILITIES shall not be available
        assert stderr.read() != ""
