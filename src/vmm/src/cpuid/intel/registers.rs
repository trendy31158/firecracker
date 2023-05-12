// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use bit_fields::bitfield;

// -------------------------------------------------------------------------------------------------
// Leaf 3
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf3Eax, u32, {
    // Reserved.
});

bitfield!(Leaf3Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf3Ecx, u32, {
    /// Bits 00 - 31 of 96 bit processor serial number. (Available in Pentium III processor only;
    /// otherwise, the value in this register is reserved.)
    bit_processor_serial_number_00_31: 0..32,
});

bitfield!(Leaf3Edx, u32, {
    /// Bits 32 - 63 of 96 bit processor serial number. (Available in Pentium III processor only;
    /// otherwise, the value in this register is reserved.)
    bit_processor_serial_number_32_63: 0..32,
});
// -------------------------------------------------------------------------------------------------
// Leaf 4
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf4Eax, u32, {
    /// Cache Type Field.
    /// - 0 = Null - No more caches.
    /// - 1 = Data Cache.
    /// - 2 = Instruction Cache.
    /// - 3 = Unified Cache.
    /// - 4-31 = Reserved.
    cache_type_field: 0..5,
    /// Cache Level (starts at 1).
    cache_level: 5..8,
    /// Self Initializing cache level (does not need SW initialization).
    sicl: 8,
    /// Fully Associative cache.
    fac: 9,
    // Reserved 10..14
    /// Maximum number of addressable IDs for logical processors sharing this cache.
    /// - Add one to the return value to get the result.
    /// - The nearest power-of-2 integer that is not smaller than (1 + EAX[25:14]) is the number of
    ///   unique initial APIC IDs reserved for addressing different logical processors sharing this
    ///   cache.
    max_num_addressable_ids_for_logical_processors_sharing_this_cache: 14..26,
    /// Maximum number of addressable IDs for processor cores in the physical package.
    /// - Add one to the return value to get the result.
    /// - The nearest power-of-2 integer that is not smaller than (1 + EAX[31:26]) is the number of
    ///   unique Core_IDs reserved for addressing different processor cores in a physical package.
    ///   Core ID is a subset of bits of the initial APIC ID.
    /// - The returned value is constant for valid initial values in ECX. Valid ECX values start
    ///   from 0.
    max_num_addressable_ids_for_processor_cores_in_physical_package: 26..32,
});

bitfield!(Leaf4Ebx, u32, {
    /// L = System Coherency Line Size.
    ///
    /// Add one to the return value to get the result.
    system_coherency_line_size: 0..12,
    /// P = Physical Line partitions.
    ///
    /// Add one to the return value to get the result.
    physical_line_partitions: 12..22,
    /// W = Ways of associativity.
    ///
    /// Add one to the return value to get the result.
    ways_of_associativity: 22..32
});

bitfield!(Leaf4Ecx, u32, {
    /// S = Number of Sets.
    ///
    /// Add one to the return value to get the result.
    number_of_sets: 0..32,
});

bitfield!(Leaf4Edx, u32, {
    /// Write-Back Invalidate/Invalidate.
    /// - 0 = WBINVD/INVD from threads sharing this cache acts upon lower level caches for threads
    ///   sharing this cache.
    /// - 1 = WBINVD/INVD is not guaranteed to act upon lower level caches of non-originating
    ///   threads sharing this cache.
    write_back_invalidate: 0,
    /// Cache Inclusiveness.
    /// - 0 = Cache is not inclusive of lower cache levels.
    /// - 1 = Cache is inclusive of lower cache levels.
    cache_inclusiveness: 1,
    /// Complex Cache Indexing.
    /// - 0 = Direct mapped cache.
    /// - 1 = A complex function is used to index the cache, potentially using all address bits.
    complex_cache_indexing: 2,
});
// -------------------------------------------------------------------------------------------------
// Leaf 5
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf5Eax, u32, {
    /// Smallest monitor-line size in bytes (default is processor's monitor granularity).
    smallest_monitor_line_size: 0..16,
    // Reserved
});

bitfield!(Leaf5Ebx, u32, {
    /// Largest monitor-line size in bytes (default is processor's monitor granularity).
    largest_monitor_line_size: 0..16,
    // Reserved
});

bitfield!(Leaf5Ecx, u32, {
    /// Enumeration of Monitor-Mwait extensions (beyond EAX and EBX registers) supported.
    enum_monitor_mwait_ext: 0,
    /// Supports treating interrupts as break-event for MWAIT, even when interrupts disabled.
    support_treating_interrupts_as_break_events_for_mwait: 1,
    // Reserved
});

bitfield!(Leaf5Edx, u32, {
    /// Number of C0* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c0_states: 0..4,
    /// Number of C1* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c1_states: 4..8,
    /// Number of C2* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c2_states: 8..12,
    /// Number of C3* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c3_states: 12..16,
    /// Number of C4* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c4_states: 16..20,
    /// Number of C5* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c5_states: 20..24,
    /// Number of C6* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c6_states: 24..28,
    /// Number of C7* sub C-states supported using MWAIT.
    ///
    /// The definition of C0 through C7 states for MWAIT extension are processor-specific C-states,
    /// not ACPI Cstates.
    c7_states: 28..32,
});
// -------------------------------------------------------------------------------------------------
// Leaf 6
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf6Eax, u32, {
    /// Digital temperature sensor is supported if set.
    digital_temperature_sensor: 0,
    /// Intel Turbo Boost Technology available (see description of IA32_MISC_ENABLE[38]).
    intel_turbo_boost_technology: 1,
    /// ARAT. APIC-Timer-always-running feature is supported if set.
    arat: 2,
    // Reserved
    /// PLN. Power limit notification controls are supported if set.
    pln: 4,
    /// ECMD. Clock modulation duty cycle extension is supported if set.
    ecmd: 5,
    /// PTM. Package thermal management is supported if set.
    ptm: 6,
    /// HWP. HWP base registers (IA32_PM_ENABLE[bit 0], IA32_HWP_CAPABILITIES, IA32_HWP_REQUEST,
    /// IA32_HWP_STATUS) are supported if set.
    hwp: 7,
    /// HWP_Notification. IA32_HWP_INTERRUPT MSR is supported if set.
    hwp_notification: 8,
    /// HWP_Activity_Window. IA32_HWP_REQUEST[bits 41:32] is supported if set.
    hwp_activity_window: 9,
    /// HWP_Energy_Performance_Preference. IA32_HWP_REQUEST[bits 31:24] is supported if set.
    hwp_energy_performance: 10,
    /// HWP_Package_Level_Request. IA32_HWP_REQUEST_PKG MSR is supported if set.
    hwp_package_level_request: 11,
    // Reserved
    /// HDC. HDC base registers IA32_PKG_HDC_CTL, IA32_PM_CTL1, IA32_THREAD_STALL MSRs are supported
    /// if set.
    hdc: 13,
    /// Intel® Turbo Boost Max Technology 3.0 available.
    intel_turbo_boost_max_technology_3: 14,
    /// HWP Capabilities. Highest Performance change is supported if set.
    hwp_capabilities: 15,
    /// HWP PECI override is supported if set.
    hwp_peci_override: 16,
    /// Flexible HWP is supported if set.
    flexible_hwp: 17,
    // Fast access mode for the IA32_HWP_REQUEST MSR is supported if set.
    fast_access_mode_for_i32_hwp_request_msr: 18,
    /// HW_FEEDBACK. IA32_HW_FEEDBACK_PTR MSR, IA32_HW_FEEDBACK_CONFIG MSR,
    /// IA32_PACKAGE_THERM_STATUS MSR bit 26, and IA32_PACKAGE_THERM_INTERRUPT MSR bit 25 are
    /// supported if set.
    hw_feedback: 19,
    // Ignoring Idle Logical Processor HWP request is supported if set.
    iilp_hwp_r: 20,
    // Reserved 21..=22
    /// Intel® Thread Director supported if set. IA32_HW_FEEDBACK_CHAR and
    /// IA32_HW_FEEDBACK_THREAD_CONFIG MSRs are supported if set.
    intel_thread_director: 23,
    // Reserved 24..=31

});

bitfield!(Leaf6Ebx, u32, {
    /// Number of Interrupt Thresholds in Digital Thermal Sensor.
    number_of_interrupt_thresholds_in_digital_thermal_sensor: 0..4,
    // Reserved 4..=31
});

bitfield!(Leaf6Ecx, u32, {
    /// Hardware Coordination Feedback Capability (Presence of IA32_MPERF and IA32_APERF). The
    /// capability to provide a measure of delivered processor performance (since last reset of the
    /// counters), as a percentage of the expected processor performance when running at the TSC
    /// frequency.
    hardware_coordination_feedback_capability: 0,
    // Reserved 1..=2
    /// The processor supports performance-energy bias preference if CPUID.06H:ECX.SETBH[bit 3] is
    /// set and it also implies the presence of a new architectural MSR called IA32_ENERGY_PERF_BIAS
    /// (1B0H).
    performance_energy_bias: 3,
    /// Reserved 04..=07
    /// Number of Intel® Thread Director classes supported by the processor. Information for that
    /// many classes is written into the Intel Thread Director Table by the hardware.
    intel_thread_director_classes: 8..16,
    // Reserved 16..=31
});

bitfield!(Leaf6Edx, u32, {
    /// Bitmap of supported hardware feedback interface capabilities.
    /// - 0 = When set to 1, indicates support for performance capability reporting.
    /// - 1 = When set to 1, indicates support for energy efficiency capability reporting.
    /// - 2-7 = Reserved
    bitmap_hardware_feedback_interface_capabilities: 0..8,
    /// Enumerates the size of the hardware feedback interface structure in number of 4 KB pages;
    /// add one to the return value to get the result.
    enum_hardware_feedback_interface_4k: 8..12,
    /// Index (starting at 0) of this logical processor's row in the hardware feedback interface
    /// structure. Note that on some parts the index may be same for multiple logical processors. On
    /// some parts the indices may not be contiguous, i.e., there may be unused rows in the hardware
    /// feedback interface structure.
    index: 16..32
});
// -------------------------------------------------------------------------------------------------
// Leaf 7
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf7Subleaf0Eax, u32, {
    /// Reports the maximum input value for supported leaf 7 sub-leaves.
    max_input_value_subleaf: 0..32
});

bitfield!(Leaf7Subleaf0Ebx, u32, {
    /// FSGSBASE. Supports RDFSBASE/RDGSBASE/WRFSBASE/WRGSBASE if 1.
    fsgsbase: 0,
    /// IA32_TSC_ADJUST MSR is supported if 1.
    ia32_tsc_adjust_msr: 1,
    /// SGX. Supports Intel® Software Guard Extensions (Intel® SGX Extensions) if 1.
    sgx: 2,
    /// BMI1.
    bmi1: 3,
    /// HLE.
    hle: 4,
    /// AVX2.
    avx2: 5,
    /// FDP_EXCPTN_ONLY. x87 FPU Data Pointer updated only on x87 exceptions if 1.
    fdp_excptn_only: 6,
    /// SMEP. Supports Supervisor-Mode Execution Prevention if 1.
    smep: 7,
    /// BMI2.
    bmi2: 8,
    /// Supports Enhanced REP MOVSB/STOSB if 1.
    suports_enhanced_rep_movsb_stosb: 9,
    /// INVPCID. If 1, supports INVPCID instruction for system software that manages process-context
    /// identifiers.
    invpcid: 10,
    /// RTM.
    rtm: 11,
    /// RDT-M. Supports Intel® Resource Director Technology (Intel® RDT) Monitoring capability if 1.
    rdt_m: 12,
    /// Deprecates FPU CS and FPU DS values if 1.
    deprecates_fpu_cs_and_fpu_ds: 13,
    /// MPX. Supports Intel® Memory Protection Extensions if 1.
    mpx: 14,
    /// RDT-A. Supports Intel® Resource Director Technology (Intel® RDT) Allocation capability if 1.
    rdt_t: 15,
    /// AVX512F.
    avx512f: 16,
    /// AVX512DQ.
    avx512dq: 17,
    /// RDSEED.
    rdseed: 18,
    /// ADX.
    adx: 19,
    /// SMAP. Supports Supervisor-Mode Access Prevention (and the CLAC/STAC instructions) if 1.
    smap: 20,
    /// AVX512_IFMA.
    avx512_ifma: 21,
    // Reserved
    /// CLFLUSHOPT.
    clfushopt: 23,
    /// CLWB.
    clwb: 24,
    /// Intel Processor Trace.
    intel_processor_trace: 25,
    /// AVX512PF. (Intel® Xeon Phi™ only.)
    avx512pf: 26,
    /// AVX512ER. (Intel® Xeon Phi™ only.)
    avx512er: 27,
    /// AVX512CD.
    avx512cd: 28,
    /// SHA. supports Intel® Secure Hash Algorithm Extensions (Intel® SHA Extensions) if 1.
    sha: 29,
    /// AVX512BW.
    avx512bw: 30,
    /// AVX512VL.
    avx512vl: 31
});

bitfield!(Leaf7Subleaf0Ecx, u32, {
    /// PREFETCHWT1. (Intel® Xeon Phi™ only.)
    prefetchwt1: 0,
    /// AVX512_VBMI.
    avx512_vbmi: 1,
    /// UMIP. Supports user-mode instruction prevention if 1.
    umip: 2,
    /// PKU. Supports protection keys for user-mode pages if 1.
    pku: 3,
    /// OSPKE. If 1, OS has set CR4.PKE to enable protection keys (and the RDPKRU/WRPKRU instructions).
    ospke: 4,
    /// WAITPKG.
    waitpkg: 5,
    /// AVX512_VBMI2.
    avx512_vbmi2: 6,
    /// CET_SS. Supports CET shadow stack features if 1. Processors that set this bit define bits
    /// 1:0 of the IA32_U_CET and IA32_S_CET MSRs. Enumerates support for the following MSRs:
    /// IA32_INTERRUPT_SPP_TABLE_ADDR, IA32_PL3_SSP, IA32_PL2_SSP, IA32_PL1_SSP, and IA32_PL0_SSP.
    cet_ss: 7,
    /// GFNI.
    gfni: 8,
    /// VAES.
    vaes: 9,
    /// VPCLMULQDQ.
    vpclmulqdq: 10,
    /// AVX512_VNNI.
    avx512_vnni: 11,
    /// AVX512_BITALG.
    avx512_bitalg: 12,
    /// TME_EN. If 1, the following MSRs are supported: IA32_TME_CAPABILITY, IA32_TME_ACTIVATE,
    /// IA32_TME_EXCLUDE_MASK, and IA32_TME_EXCLUDE_BASE.
    tme_en: 13,
    /// AVX512_VPOPCNTDQ.
    avx512_vpopcntdq: 14,
    // Reserved
    /// LA57. Supports 57-bit linear addresses and five-level paging if 1.
    la57: 16,
    /// The value of MAWAU used by the BNDLDX and BNDSTX instructions in 64-bit mode.
    value_of_mawau: 17..22,
    /// RDPID and IA32_TSC_AUX are available if 1.
    rdpid_and_ia32_tsc_aux: 22,
    /// KL. Supports Key Locker if 1.
    kl: 23,
    // Reserved
    /// CLDEMOTE. Supports cache line demote if 1.
    cldemote: 25,
    // Reserved
    /// MOVDIRI. Supports MOVDIRI if 1.
    movdiri: 27,
    /// MOVDIR64B. Supports MOVDIR64B if 1.
    movdiri64b: 28,
    // Reserved
    /// SGX_LC. Supports SGX Launch Configuration if 1.
    sgx_lc: 30,
    /// PKS. Supports protection keys for supervisor-mode pages if 1.
    pks: 31
});

bitfield!(Leaf7Subleaf0Edx, u32, {
    // Reserved
    /// AVX512_4VNNIW. (Intel® Xeon Phi™ only.)
    avx512_4vnniw: 2,
    /// AVX512_4FMAPS. (Intel® Xeon Phi™ only.)
    avx512_4fmaps: 3,
    /// Fast Short REP MOV.
    fast_short_rep_mov: 4,
    // Reserved 5..=7
    /// AVX512_VP2INTERSECT.
    avx512_vp2intersect: 8,
    // Reserved
    /// MD_CLEAR supported.
    md_clear: 10,
    // Reserved
    /// SERIALIZE.
    serialize:  11..14,
    /// Hybrid. If 1, the processor is identified as a hybrid part.
    hydrid: 15,
    // Reserved 16..=17
    /// PCONFIG. Supports PCONFIG if 1.
    pconfig: 18,
    // Reserved
    /// CET_IBT. Supports CET indirect branch tracking features if 1. Processors that set this bit
    /// define bits 5:2 and bits 63:10 of the IA32_U_CET and IA32_S_CET MSRs.
    cet_ibt: 19,
    // Reserved 21..=25
    /// Enumerates support for indirect branch restricted speculation (IBRS) and the indirect branch
    /// predictorn barrier (IBPB). Processors that set this bit support the IA32_SPEC_CTRL MSR and
    /// the A32_PRED_CMD MSR. They allow software to set IA32_SPEC_CTRL[0] (IBRS) and
    /// IA32_PRED_CMD[0] (IBPB).
    ibrs_ibpb_enum: 26,
    /// Enumerates support for single thread indirect branch predictors (STIBP). Processors that set
    /// this bit support the IA32_SPEC_CTRL MSR. They allow software to set IA32_SPEC_CTRL[1]
    /// (STIBP).
    stibp_enum: 27,
    /// Enumerates support for L1D_FLUSH. Processors that set this bit support the IA32_FLUSH_CMD
    /// MSR. They allow software to set IA32_FLUSH_CMD[0] (L1D_FLUSH).
    l1d_flush_enum: 28,
    /// Enumerates support for the IA32_ARCH_CAPABILITIES MSR.
    ia32_arch_capabilities_msr_enum: 29,
    /// Enumerates support for the IA32_CORE_CAPABILITIES MSR.
    ia32_core_capabilities_msr_enum: 30,
    /// Enumerates support for Speculative Store Bypass Disable (SSBD). Processors that set this bit
    /// support the IA32_SPEC_CTRL MSR. They allow software to set IA32_SPEC_CTRL[2] (SSBD).
    ssbd_enum: 31,
});

bitfield!(Leaf7Subleaf1Eax, u32, {
    // Reserved 0..=3
    /// AVX-VNNI. AVX (VEX-encoded) versions of the Vector Neural Network Instructions.
    avx_vnni: 4,
    /// AVX512_BF16. Vector Neural Network Instructions supporting BFLOAT16 inputs and conversion
    /// instructions from IEEE single precision.
    avx512_bf16: 5,
    // Reserved 6..=9
    /// If 1, supports fast zero-length REP MOVSB.
    fast_zero_length_rep_movsh: 10,
    /// If 1, supports fast short REP STOSB.
    fast_short_rep_stosb: 11,
    /// If 1, supports fast short REP CMPSB, REP SCASB.
    fast_short_rep_cmpsb_rep_scasb: 12,
    // Reserved 13..=21
    /// HRESET. If 1, supports history reset via the HRESET instruction and the IA32_HRESET_ENABLE
    /// MSR. When set, indicates that the Processor History Reset Leaf (EAX = 20H) is valid.
    hreset: 22,
    // Reserved 23..=31
});

bitfield!(Leaf7Subleaf1Ebx, u32, {
    /// Enumerates the presence of the IA32_PPIN and IA32_PPIN_CTL MSRs. If 1, these MSRs are
    /// supported.
    ia32_ppin_and_ia32_ppin_ctl_msrs_enum: 0
    // Reserved 1..=31
});

bitfield!(Leaf7Subleaf1Ecx, u32, {
    // Reserved
});

bitfield!(Leaf7Subleaf1Edx, u32, {
    // Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 9
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf9Eax, u32, {
    /// Value of bits [31:0] of IA32_PLATFORM_DCA_CAP MSR (address 1F8H).
    ia32_platform_dca_cap_msr: 0..32
});

bitfield!(Leaf9Ebx, u32, {
    // Reserved
});

bitfield!(Leaf9Ecx, u32, {
    // Reserved
});

bitfield!(Leaf9Edx, u32, {
    // Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf A
// -------------------------------------------------------------------------------------------------

bitfield!(LeafAEax, u32, {
    /// Version ID of architectural performance monitoring.
    version_id_of_architectural_performance_monitoring: 0..8,
    /// Number of general-purpose performance monitoring counter per logical processor.
    num_perf_monitor_counter_per_logical_processor: 8..16,
    /// Bit width of general-purpose, performance monitoring counter.
    bot_width_perf_monitor_counter: 16..24,
    /// Length of EBX bit vector to enumerate architectural performance monitoring events.
    /// Architectural event x is supported if EBX[x]=0 && EAX[31:24]>x.
    len_ebx_bit_vec: 24..32
});

bitfield!(LeafAEbx, u32, {
    /// Core cycle event not available if 1 or if EAX[31:24]<1.
    core_cycle_event: 0,
    /// Instruction retired event not available if 1 or if EAX[31:24]<2.
    instruction_retired_event: 1,
    /// Reference cycles event not available if 1 or if EAX[31:24]<3.
    reference_cycles_event: 2,
    /// Last-level cache reference event not available if 1 or if EAX[31:24]<4.
    last_level_cache_reference_event: 3,
    /// Last-level cache misses event not available if 1 or if EAX[31:24]<5.
    last_level_cache_misses_event: 4,
    /// Branch instruction retired event not available if 1 or if EAX[31:24]<6.
    branch_instruction_retired_event: 5,
    /// Branch mispredict retired event not available if 1 or if EAX[31:24]<7.
    branch_mispredict_retired_event: 6,
    /// Top-down slots event not available if 1 or if EAX[31:24]<8.
    top_down_slots_event: 7,
    // Reserved 8..=31
});

bitfield!(LeafAEcx, u32, {
    /// Supported fixed counters bit mask. Fixed-function performance counter 'i' is supported if
    /// bit ‘i’ is 1 (first counter index starts at zero). It is recommended to use the following
    /// logic to determine if a Fixed Counter is supported:
    /// FxCtr[i]_is_supported := ECX[i] || (EDX[4:0] > i);
    supported_fixed_counters_bit_mask: 0..32
});

bitfield!(LeafAEdx, u32, {
    /// Number of contiguous fixed-function performance counters starting from 0 (if Version ID >1).
    contigous_fixed_function_performance_counter: 0..5,
    /// Bit width of fixed-function performance counters (if Version ID > 1).
    bit_width_of_fixed_function_performnace_counter: 5..13,
    // Reserved 13..=14
    /// AnyThread deprecation.
    anythread_deprecation: 15
    // Reserved 16..=31
});
// -------------------------------------------------------------------------------------------------
// Leaf D
// -------------------------------------------------------------------------------------------------
// Leaf 0

bitfield!(LeafDSubleaf0Eax, u32, {
    // Bits 31 - 00: Reports the supported bits of the lower 32 bits of XCR0. XCR0[n] can be set to
    // 1 only if EAX[n] is 1.
    /// x87 state.
    x86_state: 0,
    /// SSE state.
    sse_state: 1,
    /// AVX state.
    avx_state: 2,
    /// MPX state.
    mpx_state: 3..5,
    /// AVX-512 state.
    avx512_state: 5..8,
    /// Used for IA32_XSS.
    used_for_ia32_xss: 8,
    /// PKRU state.
    pkru_state: 9,
    // Reserved 10..=12
    /// Used for IA32_XSS.
    used_for_ia32_xss_1: 13,
    // Reserved 14..=15
    /// Used for IA32_XSS.
    used_for_ia32_xss_2: 16,
    // Reserved 17..=31
});

bitfield!(LeafDSubleaf0Ebx, u32, {
    /// Maximum size (bytes, from the beginning of the XSAVE/XRSTOR save area) required by enabled
    /// features in XCR0. May be different than ECX if some features at the end of the XSAVE save
    /// area are not enabled.
    maximum_size: 0..32
});

bitfield!(LeafDSubleaf0Ecx, u32, {
    /// Maximum size (bytes, from the beginning of the XSAVE/XRSTOR save area) of the XSAVE/XRSTOR
    /// save area required by all supported features in the processor, i.e., all the valid bit
    /// fields in XCR0.
    ///
    // `LeafDSubleaf0Ecx::maximum_size() >= LeafDSubleaf0Ebx::maximum_size()`
    maximum_size: 0..32
});

bitfield!(LeafDSubleaf0Edx, u32, {
    // Reports the supported bits of the upper 32 bits of XCR0. XCR0[n+32] can be set to 1 only if
    // EDX[n] is 1.
    // Reserved
});
// Leaf 1

bitfield!(LeafDSubleaf1Eax, u32, {
    /// XSAVEOPT is available.
    xsaveopt_available: 0,
    /// Supports XSAVEC and the compacted form of XRSTOR if set.
    xsavec_compacted_xrstor: 1,
    /// Supports XGETBV with ECX = 1 if set.
    xgetbv: 2,
    /// Supports XSAVES/XRSTORS and IA32_XSS if set.
    xsaves_xrstors_ia32_xss: 3,
    // Reserved 0..32
});

bitfield!(LeafDSubleaf1Ebx, u32, {
    /// The size in bytes of the XSAVE area containing all states enabled by XCRO | IA32_XSS.
    xsave_size: 0..32,
});

bitfield!(LeafDSubleaf1Ecx, u32, {
    // Reports the supported bits of the lower 32 bits of the IA32_XSS MSR. IA32_XSS[n] can be set
    // to 1 only if ECX[n] is 1.
    /// Used for XCR0.
    xcr0_1: 0..8,
    /// PT state.
    pt_state: 8,
    /// Used for XCR0.
    xcr0_2: 9,
    // Reserved
    /// CET user state.
    cet_user_state: 11,
    /// CET supervisor state.
    cet_supervisor_state: 12,
    /// HDC state.
    hdc_state: 13,
    // Reserved
    /// LBR state (architectural).
    lbr_state: 15,
    /// HWP state.
    hwp_state: 16,
    // Reserved 17..=31
});

bitfield!(LeafDSubleaf1Edx, u32, {
    // Reports the supported bits of the upper 32 bits of the IA32_XSS MSR. IA32_XSS[n+32] can be
    // set to 1 only if EDX[n] is 1.
    // Reserved
});
// Leaf >1

bitfield!(LeafDSubleafGt1Eax, u32, {
    /// The size in bytes (from the offset specified in EBX) of the save area for an extended state
    /// feature associated with a valid sub-leaf index, n.
    save_area_size: 0..32,
});

bitfield!(LeafDSubleafGt1Ebx, u32, {
    /// The offset in bytes of this extended state component’s save area from the beginning of the
    /// XSAVE/XRSTOR area.
    ///
    /// This field reports 0 if the sub-leaf index, n, does not map to a valid bit in the XCR0
    /// register*.
    ///
    /// *If ECX contains an invalid sub-leaf index, EAX/EBX/ECX/EDX return 0. Sub-leaf n
    /// (0 ≤ n ≤ 31) is invalid if sub-leaf 0 returns 0 in EAX[n] and sub-leaf 1 returns 0 in
    /// ECX[n]. Sub-leaf n (32 ≤ n ≤ 63) is invalid if sub-leaf 0 returns 0 in EDX[n-32] and
    /// sub-leaf 1 returns 0 in EDX[n-32].
    save_area_offset: 0..32
});

bitfield!(LeafDSubleafGt1Ecx, u32, {
    /// Is set if the bit n (corresponding to the sub-leaf index) is supported in the IA32_XSS MSR;
    /// it is clear if bit n is instead supported in XCR0.
    ///
    /// This field reports 0 if the sub-leaf index, n, is invalid*.
    ///
    /// *If ECX contains an invalid sub-leaf index, EAX/EBX/ECX/EDX return 0. Sub-leaf n
    /// (0 ≤ n ≤ 31) is invalid if sub-leaf 0 returns 0 in EAX[n] and sub-leaf 1 returns 0 in
    /// ECX[n]. Sub-leaf n (32 ≤ n ≤ 63) is invalid if sub-leaf 0 returns 0 in EDX[n-32] and
    /// sub-leaf 1 returns 0 in EDX[n-32].
    supported_ia32_xss_msr: 0,
    /// Is set if, when the compacted format of an XSAVE area is used, this extended state component
    /// located on the next 64-byte boundary following the preceding state component (otherwise, it
    /// is located immediately following the preceding state component).
    ///
    /// This field reports 0 if the sub-leaf index, n, is invalid*.
    ///
    /// *If ECX contains an invalid sub-leaf index, EAX/EBX/ECX/EDX return 0. Sub-leaf n
    /// (0 ≤ n ≤ 31) is invalid if sub-leaf 0 returns 0 in EAX[n] and sub-leaf 1 returns 0 in
    /// ECX[n]. Sub-leaf n (32 ≤ n ≤ 63) is invalid if sub-leaf 0 returns 0 in EDX[n-32] and
    /// sub-leaf 1 returns 0 in EDX[n-32].
    compacted_xsave_used: 1,
    // 0..=31 reserved
});

bitfield!(LeafDSubleafGt1Edx, u32, {
    // This field reports 0 if the sub-leaf index, n, is invalid*; otherwise it is reserved.
    // 0..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf F
// -------------------------------------------------------------------------------------------------
// Leaf 0

bitfield!(LeafFSubleaf0Eax, u32, {
    // Reserved
});

bitfield!(LeafFSubleaf0Ebx, u32, {
    /// Maximum range (zero-based) of RMID within this physical processor of all types.
    max_rmid_range: 0..32,
});

bitfield!(LeafFSubleaf0Ecx, u32, {
    // Reserved
});

bitfield!(LeafFSubleaf0Edx, u32, {
    // Reserved
    /// Supports L3 Cache Intel RDT Monitoring if 1.
    l3_rdt_monitor: 1,
    // 2..=32 reserved
});
// Leaf 1

bitfield!(LeafFSubleaf1Eax, u32, {
    // Reserved
});

bitfield!(LeafFSubleaf1Ebx, u32, {
    /// Conversion factor from reported IA32_QM_CTR value to occupancy metric (bytes) and Memory
    /// Bandwidth Monitoring (MBM) metrics.
    ia32_qm_ctr_conv_factor: 0..32,
});

bitfield!(LeafFSubleaf1Ecx, u32, {
    /// Maximum range (zero-based) of RMID of this resource type.
    rmid_max: 0..32,
});

bitfield!(LeafFSubleaf1Edx, u32, {
    /// Supports L3 occupancy monitoring if 1.
    l3_occupancy_monitor: 0,
    /// Supports L3 Total Bandwidth monitoring if 1.
    l3_total_band_monitor: 1,
    /// Supports L3 Local Bandwidth monitoring if 1.
    l3_local_band_monitor: 2,
    // 0..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 10
// -------------------------------------------------------------------------------------------------
// Leaf 0

bitfield!(Leaf10Subleaf0Eax, u32, {
    // Reserved
});

bitfield!(Leaf10Subleaf0Ebx, u32, {
    // Reserved
    /// Supports L3 Cache Allocation Technology if 1.
    l3_alloc: 1,
    /// Supports L2 Cache Allocation Technology if 1.
    l2_alloc: 2,
    /// Supports Memory Bandwidth Allocation if 1.
    mem_band_alloc: 3,
    // 04..=31 reserved
});

bitfield!(Leaf10Subleaf0Ecx, u32, {
    // Reserved
});

bitfield!(Leaf10Subleaf0Edx, u32, {
    // Reserved
});
// Leaf 1

bitfield!(Leaf10Subleaf1Eax, u32, {
    /// Length of the capacity bit mask for the corresponding ResID. Add one to the return value to
    /// get the result.
    len_cap_resid_mask: 0..5,
    // 5..=31 reserved
});

bitfield!(Leaf10Subleaf1Ebx, u32, {
    /// Bit-granular map of isolation/contention of allocation units.
    granular_iso_cont_map: 0..32,
});

bitfield!(Leaf10Subleaf1Ecx, u32, {
    // 0..=1 reserved
    /// Code and Data Prioritization Technology supported if 1.
    cd_prior: 2,
    // 3..=31 reserved
});

bitfield!(Leaf10Subleaf1Edx, u32, {
    /// Highest COS number supported for this ResID.
    highest_cos_resid: 0..16,
    // 0..=31 reserved
});
// Leaf 2

bitfield!(Leaf10Subleaf2Eax, u32, {
    /// Length of the capacity bit mask for the corresponding ResID. Add one to the return value to
    /// get the result.
    len_cap_resid_mask: 0..5,
    // 5..=31 reserved
});

bitfield!(Leaf10Subleaf2Ebx, u32, {
    /// Bit-granular map of isolation/contention of allocation units.
    granular_iso_cont_map: 0..32,
});

bitfield!(Leaf10Subleaf2Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf10Subleaf2Edx, u32, {
    /// Highest COS number supported for this ResID.
    highest_cos_resid: 0..16,
    // 0..=31 reserved
});
// Leaf 3

bitfield!(Leaf10Subleaf3Eax, u32, {
    /// Reports the maximum MBA throttling value supported for the corresponding ResID. Add one to
    /// the return value to get the result.
    max_mba_throt_resid: 0..12,
    // reserved 12..=31
});

bitfield!(Leaf10Subleaf3Ebx, u32, {
    // Reserved
});

bitfield!(Leaf10Subleaf3Ecx, u32, {
    // 0..=1 reserved
    /// Reports whether the response of the delay values is linear.
    linear_response_delay_values: 2,
    // 3..=31 reserved
});

bitfield!(Leaf10Subleaf3Edx, u32, {
    /// Highest COS number supported for this ResID.
    highest_cos_resid: 0..16,
    // 16..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 12
// -------------------------------------------------------------------------------------------------
// Leaf 0

bitfield!(Leaf12Subleaf0Eax, u32, {
    /// SGX1. If 1, Indicates Intel SGX supports the collection of SGX1 leaf functions.
    sgx1: 0,
    /// SGX2. If 1, Indicates Intel SGX supports the collection of SGX2 leaf functions.
    sgx2: 1,
    // 2..=4 reserved
    /// If 1, indicates Intel SGX supports ENCLV instruction leaves EINCVIRTCHILD, EDECVIRTCHILD,
    /// and ESETCONTEXT.
    enclv: 5,
    /// If 1, indicates Intel SGX supports ENCLS instruction leaves ETRACKC, ERDINFO, ELDBC, and
    /// ELDUC.
    encls: 6,
    // 7..=31 reserved
});

bitfield!(Leaf12Subleaf0Ebx, u32, {
    /// MISCSELECT. Bit vector of supported extended SGX features.
    miscselect: 0..32,
});

bitfield!(Leaf12Subleaf0Ecx, u32, {
    // Reserved
});

bitfield!(Leaf12Subleaf0Edx, u32, {
    /// MaxEnclaveSize_Not64. The maximum supported enclave size in non-64-bit mode is 2^(EDX[7:0]).
    max_enclave_size_not_64: 0..8,
    /// MaxEnclaveSize_64. The maximum supported enclave size in 64-bit mode is 2^(EDX[15:8]).
    max_enclave_size_64: 8..16,
    // 16..=31 reserved
});
// Leaf 1

bitfield!(Leaf12Subleaf1Eax, u32, {
    /// Reports the valid bits of SECS.ATTRIBUTES[31:0] that software can set with ECREATE.
    ecreate_attrs_0_31: 0..32,
});

bitfield!(Leaf12Subleaf1Ebx, u32, {
    /// Reports the valid bits of SECS.ATTRIBUTES[63:32] that software can set with ECREATE.
    ecreate_attrs_32_63: 0..32,
});

bitfield!(Leaf12Subleaf1Ecx, u32, {
    /// Reports the valid bits of SECS.ATTRIBUTES[95:64] that software can set with ECREATE.
    ecreate_attrs_64_95: 0..32,
});

bitfield!(Leaf12Subleaf1Edx, u32, {
    /// Reports the valid bits of SECS.ATTRIBUTES[127:96] that software can set with ECREATE.
    ecreate_attrs_96_127: 0..32,
});
// Leaf >1

bitfield!(Leaf12SubleafGt1Eax, u32, {
    /// Sub-leaf Type
    /// - 0000b: Indicates this sub-leaf is invalid.
    /// - 0001b: This sub-leaf enumerates an EPC section. EBX:EAX and EDX:ECX provide information on the
    /// Enclave Page Cache (EPC) section.
    /// All other type encodings are reserved.
    subleaf_type: 0..4,
    /// Bits 31:12 of the physical address of the base of the EPC section.
    ///
    /// When EAX[03:00] = 0001b (otherwise 0)
    epc_base_31_12: 12..32,
});

bitfield!(Leaf12SubleafGt1Ebx, u32, {
    /// Bits 51:32 of the physical address of the base of the EPC section.
    ///
    /// When EAX[03:00] = 0001b (otherwise 0)
    epc_base_51_32: 0..20,
    // 20..=31 reserved
});

bitfield!(Leaf12SubleafGt1Ecx, u32, {
    /// EPC section property encoding defined as follows:
    /// - If ECX[3:0] = 0000b, then all bits of the EDX:ECX pair are enumerated as 0.
    /// - If ECX[3:0] = 0001b, then this section has confidentiality and integrity protection.
    /// - If ECX[3:0] = 0010b, then this section has confidentiality protection only.
    /// All other encodings are reserved.
    ///
    /// When EAX[03:00] = 0001b (otherwise 0)
    epc_section: 0..4,
    // 4..=11 reserved
    /// Bits 31:12 of the size of the corresponding EPC section within the Processor Reserved
    /// Memory.
    ///
    /// When EAX[03:00] = 0001b (otherwise 0)
    epc_reserved_31_12: 12..32,

});

bitfield!(Leaf12SubleafGt1Edx, u32, {
    /// Bits 51:32 of the size of the corresponding EPC section within the Processor Reserved
    /// Memory.
    ///
    /// When EAX[03:00] = 0001b (otherwise 0)
    epc_reserved_51_32: 0..20,
    // 20..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 14
// -------------------------------------------------------------------------------------------------
// Leaf 0

bitfield!(Leaf14Subleaf0Eax, u32, {
    /// Reports the maximum sub-leaf supported in leaf 14H.
    ///
    /// **At the moment of writing the Intel specification only notes the format of ECX=1, therefore
    /// this field should only be 0 or 1**
    max_subleaf: 0..32,
});

bitfield!(Leaf14Subleaf0Ebx, u32, {
    /// If 1, indicates that IA32_RTIT_CTL.CR3Filter can be set to 1, and that IA32_RTIT_CR3_MATCH
    /// MSR can be accessed.
    ia32_rtit: 0,
    /// If 1, indicates support of Configurable PSB and Cycle-Accurate Mode.
    psb_config_cam: 1,
    /// If 1, indicates support of IP Filtering, TraceStop filtering, and preservation of Intel PT
    /// MSRs across warm reset.
    ip_filtering_and_ts_filtering_and_pt_msr_preservation: 2,
    /// If 1, indicates support of MTC timing packet and suppression of COFI-based packets.
    mtc_timing_and_cofi_suppression: 3,
    /// If 1, indicates support of PTWRITE. Writes can set IA32_RTIT_CTL[12] (PTWEn) and
    /// IA32_RTIT_CTL[5] (FUPonPTW), and PTWRITE can generate packets.
    ptwrite: 4,
    /// If 1, indicates support of Power Event Trace. Writes can set IA32_RTIT_CTL[4] (PwrEvtEn),
    /// enabling Power Event Trace packet generation.
    power_event_trace: 5,
    /// If 1, indicates support for PSB and PMI preservation. Writes can set IA32_RTIT_CTL[56]
    /// (InjectPsbPmiOnEnable), enabling the processor to set IA32_RTIT_STATUS[7] (PendTopaPMI)
    /// and/or IA32_RTIT_STATUS[6] (PendPSB) in order to preserve ToPA PMIs and/or PSBs otherwise
    /// lost due to Intel PT disable. Writes can also set PendToPAPMI and PendPSB.
    psb_and_pmi_preservation: 6,
    /// If 1, writes can set IA32_RTIT_CTL[31] (EventEn), enabling Event Trace packet generation.
    ia32_rtit_ctl_31: 7,
    /// If 1, writes can set IA32_RTIT_CTL[55] (DisTNT), disabling TNT packet generation.
    ia32_rtit_ctl_55: 8,
    // 9..=31 reserved
});

bitfield!(Leaf14Subleaf0Ecx, u32, {
    /// If 1, Tracing can be enabled with IA32_RTIT_CTL.ToPA = 1, hence utilizing the ToPA output
    /// scheme; IA32_RTIT_OUTPUT_BASE and IA32_RTIT_OUTPUT_MASK_PTRS MSRs can be accessed.
    ia32_rtit_ctl_topa: 0,
    /// If 1, ToPA tables can hold any number of output entries, up to the maximum allowed by the
    /// MaskOrTableOffset field of IA32_RTIT_OUTPUT_MASK_PTRS.
    topa_ext: 1,
    /// If 1, indicates support of Single-Range Output scheme.
    sros: 2,
    /// If 1, indicates support of output to Trace Transport subsystem.
    otts: 3,
    // 4..=30 reserved
    /// If 1, generated packets which contain IP payloads have LIP values, which include the CS base component.
    lip_cs_base: 31,
});

bitfield!(Leaf14Subleaf0Edx, u32, {
    /// Reserved
});
// Leaf 1

bitfield!(Leaf14Subleaf1Eax, u32, {
    /// Number of configurable Address Ranges for filtering.
    configurable_filterig_addr_ranges: 0..3,
    // 3..=15 reserved
    /// Bitmap of supported MTC period encodings.
    mtc_period_encodings: 16..32,
});

bitfield!(Leaf14Subleaf1Ebx, u32, {
    /// Bitmap of supported Cycle Threshold value encodings.
    cycle_threshold_value_encodings: 0..16,
    /// Bitmap of supported Configurable PSB frequency encodings.
    configurable_psb_freq_encodings: 16..32,
});

bitfield!(Leaf14Subleaf1Ecx, u32, {
    // Reserved
});

bitfield!(Leaf14Subleaf1Edx, u32, {
    // Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 15
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf15Eax, u32, {
    /// An unsigned integer which is the denominator of the TSC/”core crystal clock” ratio.
    ///
    /// If EBX[31:0] is 0, the TSC/”core crystal clock” ratio is not enumerated. EBX[31:0]/EAX[31:0]
    /// indicates the ratio of the TSC frequency and the core crystal clock frequency. If ECX is 0,
    /// the nominal core crystal clock frequency is not enumerated. “TSC frequency” = “core crystal
    /// clock frequency” * EBX/EAX. The core crystal clock may differ from the reference clock, bus
    /// clock, or core clock frequencies.
    tsc_denominator: 0..32,

});

bitfield!(Leaf15Ebx, u32, {
    /// An unsigned integer which is the numerator of the TSC/”core crystal clock” ratio.
    ///
    /// If EBX[31:0] is 0, the TSC/”core crystal clock” ratio is not enumerated. EBX[31:0]/EAX[31:0]
    /// indicates the ratio of the TSC frequency and the core crystal clock frequency. If ECX is 0,
    /// the nominal core crystal clock frequency is not enumerated. “TSC frequency” = “core crystal
    /// clock frequency” * EBX/EAX. The core crystal clock may differ from the reference clock, bus
    /// clock, or core clock frequencies.
    tsc_numerator: 0..32,
});

bitfield!(Leaf15Ecx, u32, {
    /// An unsigned integer which is the nominal frequency of the core crystal clock in Hz.
    ///
    /// If EBX[31:0] is 0, the TSC/”core crystal clock” ratio is not enumerated. EBX[31:0]/EAX[31:0]
    /// indicates the ratio of the TSC frequency and the core crystal clock frequency. If ECX is 0,
    /// the nominal core crystal clock frequency is not enumerated. “TSC frequency” = “core crystal
    /// clock frequency” * EBX/EAX. The core crystal clock may differ from the reference clock, bus
    /// clock, or core clock frequencies.
    nominal_frequency: 0..32
});

bitfield!(Leaf15Edx, u32, {
    /// Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 16
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf16Eax, u32, {
    /// Processor Base Frequency (in MHz).
    base_frequency: 0..15,
    // 15..=31 reserved
});

bitfield!(Leaf16Ebx, u32, {
    /// Maximum Frequency (in MHz).
    max_frequency: 0..15,
    // 15..=31 reserved
});

bitfield!(Leaf16Ecx, u32, {
    /// Bus (Reference) Frequency (in MHz).
    ref_frequency: 0..15,
    // 15..=31 reserved
});

bitfield!(Leaf16Edx, u32, {
    /// Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 17
// -------------------------------------------------------------------------------------------------
// leaf 0

bitfield!(Leaf17Subleaf0Eax, u32, {
    /// MaxSOCID_Index. Reports the maximum input value of supported sub-leaf in leaf 17H.
    max_socid_index: 0..32,
});

bitfield!(Leaf17Subleaf0Ebx, u32, {
    /// SOC Vendor ID.
    soc_vendor_id: 0..16,
    /// IsVendorScheme. If 1, the SOC Vendor ID field is assigned via an industry standard
    /// enumeration scheme. Otherwise, the SOC Vendor ID field is assigned by Intel.
    is_vendor_scheme: 16,
    // 17..=31 reserved
});

bitfield!(Leaf17Subleaf0Ecx, u32, {
    /// Project ID. A unique number an SOC vendor assigns to its SOC projects.
    project_id: 0..32,
});

bitfield!(Leaf17Subleaf0Edx, u32, {
    /// Stepping ID. A unique number within an SOC project that an SOC vendor assigns.
    stepping_id: 0..32,
});
// leaf 1

bitfield!(Leaf17Subleaf1Eax, u32, {
    /// SOC Vendor Brand String. UTF-8 encoded string.
    ///
    /// Leaf 17H output depends on the initial value in ECX. SOC Vendor Brand String is a UTF-8
    /// encoded string padded with trailing bytes of 00H. The complete SOC Vendor Brand String is
    /// constructed by concatenating in ascending order of EAX:EBX:ECX:EDX and from the sub-leaf 1
    /// fragment towards sub-leaf 3.
    soc_string: 0..32,
});

bitfield!(Leaf17Subleaf1Ebx, u32, {
    /// SOC Vendor Brand String. UTF-8 encoded string.
    ///
    /// Leaf 17H output depends on the initial value in ECX. SOC Vendor Brand String is a UTF-8
    /// encoded string padded with trailing bytes of 00H. The complete SOC Vendor Brand String is
    /// constructed by concatenating in ascending order of EAX:EBX:ECX:EDX and from the sub-leaf 1
    /// fragment towards sub-leaf 3.
    soc_string: 0..32,
});

bitfield!(Leaf17Subleaf1Ecx, u32, {
    /// SOC Vendor Brand String. UTF-8 encoded string.
    ///
    /// Leaf 17H output depends on the initial value in ECX. SOC Vendor Brand String is a UTF-8
    /// encoded string padded with trailing bytes of 00H. The complete SOC Vendor Brand String is
    /// constructed by concatenating in ascending order of EAX:EBX:ECX:EDX and from the sub-leaf 1
    /// fragment towards sub-leaf 3.
    soc_string: 0..32,
});

bitfield!(Leaf17Subleaf1Edx, u32, {
    /// SOC Vendor Brand String. UTF-8 encoded string.
    ///
    /// Leaf 17H output depends on the initial value in ECX. SOC Vendor Brand String is a UTF-8
    /// encoded string padded with trailing bytes of 00H. The complete SOC Vendor Brand String is
    /// constructed by concatenating in ascending order of EAX:EBX:ECX:EDX and from the sub-leaf 1
    /// fragment towards sub-leaf 3.
    soc_string: 0..32,
});
// leaf >3

bitfield!(Leaf17SubleafGt3Eax, u32, {
    // Reserved
});

bitfield!(Leaf17SubleafGt3Ebx, u32, {
    // Reserved
});

bitfield!(Leaf17SubleafGt3Ecx, u32, {
    // Reserved
});

bitfield!(Leaf17SubleafGt3Edx, u32, {
    // Reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 18
// -------------------------------------------------------------------------------------------------
// leaf 0

bitfield!(Leaf18Subleaf0Eax, u32, {
    /// Reports the maximum input value of supported sub-leaf in leaf 18H.
    max_subleaf: 0..32,
});

bitfield!(Leaf18Subleaf0Ebx, u32, {
    /// 4K page size entries supported by this structure.
    page_4k: 0,
    /// 2MB page size entries supported by this structure.
    page_2m: 1,
    /// 4MB page size entries supported by this structure.
    page_4m: 2,
    /// 1 GB page size entries supported by this structure.
    page_3g: 3,
    // 4..=7 reserved
    /// Partitioning (0: Soft partitioning between the logical processors sharing this structure).
    partitioning: 8..16,
    // 11..=15 reserved
    /// W = Ways of associativity.
    ways_of_associativity: 16..32,
});

bitfield!(Leaf18Subleaf0Ecx, u32, {
    /// S = Number of Sets.
    number_of_sets: 0..32,
});

bitfield!(Leaf18Subleaf0Edx, u32, {
    /// Translation cache type field.
    /// - 00000b: Null (indicates this sub-leaf is not valid).
    /// - 00001b: Data TLB.
    /// - 00010b: Instruction TLB.
    /// - 00011b: Unified TLB*.
    /// - 00100b: Load Only TLB. Hit on loads; fills on both loads and stores.
    /// - 00101b: Store Only TLB. Hit on stores; fill on stores.
    /// All other encodings are reserved.
    ///
    /// * Some unified TLBs will allow a single TLB entry to satisfy data read/write and instruction
    /// fetches. Others will require separate entries (e.g., one loaded on data read/write and
    /// another loaded on an instruction fetch) . Please see the Intel® 64 and IA-32 Architectures
    /// Optimization Reference Manual for details of a particular product.
    translation_cache_type: 0..5,
    /// Translation cache level (starts at 1).
    translation_cache_level: 5..8,
    /// Fully associative structure.
    fully_associative_structure: 8,
    // 9..=13 reserved
    /// Maximum number of addressable IDs for logical processors sharing this translation cache**
    ///
    /// **Add one to the return value to get the result.
    max_addressable_ids: 14..26,
    // 26..=31 reserved
});
// leaf >0

bitfield!(Leaf18SubleafGt0Eax, u32, {
    // reserved
});

bitfield!(Leaf18SubleafGt0Ebx, u32, {
    /// 4K page size entries supported by this structure.
    page_4k: 0,
    /// 2MB page size entries supported by this structure.
    page_2m: 1,
    /// 4MB page size entries supported by this structure.
    page_4m: 2,
    /// 1 GB page size entries supported by this structure.
    page_3g: 3,
    // 4..=7 reserved
    /// Partitioning (0: Soft partitioning between the logical processors sharing this structure).
    partitioning: 8..16,
    // 11..=15 reserved
    /// W = Ways of associativity.
    ways_of_associativity: 16..32,
});

bitfield!(Leaf18SubleafGt0Ecx, u32, {
    /// S = Number of Sets.
    number_of_sets: 0..32,
});

bitfield!(Leaf18SubleafGt0Edx, u32, {
    /// Translation cache type field.
    /// - 00000b: Null (indicates this sub-leaf is not valid).
    /// - 00001b: Data TLB.
    /// - 00010b: Instruction TLB.
    /// - 00011b: Unified TLB*.
    /// - 00100b: Load Only TLB. Hit on loads; fills on both loads and stores.
    /// - 00101b: Store Only TLB. Hit on stores; fill on stores.
    /// All other encodings are reserved.
    ///
    /// * Some unified TLBs will allow a single TLB entry to satisfy data read/write and instruction
    /// fetches. Others will require separate entries (e.g., one loaded on data read/write and
    /// another loaded on an instruction fetch) . Please see the Intel® 64 and IA-32 Architectures
    /// Optimization Reference Manual for details of a particular product.
    translation_cache_type: 0..5,
    /// Translation cache level (starts at 1).
    translation_cache_level: 5..8,
    /// Fully associative structure.
    fully_associative_structure: 8,
    // 9..=13 reserved
    /// Maximum number of addressable IDs for logical processors sharing this translation cache**
    ///
    /// **Add one to the return value to get the result.
    max_addressable_ids: 14..26,
    // 26..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 19
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf19Eax, u32, {
    /// Key Locker restriction of CPL0-only supported.
    cpl0_only: 0,
    /// Key Locker restriction of no-encrypt supported.
    no_encrypt: 1,
    /// Key Locker restriction of no-decrypt supported.
    no_decrypt: 2,
    // 3..=31 reserved

});

bitfield!(Leaf19Ebx, u32, {
    /// AESKLE. If 1, the AES Key Locker instructions are fully enabled.
    aeskle: 0,
    // Reserved.
    /// If 1, the AES wide Key Locker instructions are supported.
    aes_wide_key_locker: 2,
    // Reserved
    /// If 1, the platform supports the Key Locker MSRs (IA32_COPY_LOCAL_TO_PLATFORM,
    /// IA23_COPY_PLATFORM_TO_LOCAL, IA32_COPY_STATUS, and IA32_IWKEYBACKUP_STATUS) and backing up
    /// the internal wrapping key.
    key_locker_msrs: 4,
    // 5..=31 reserved
});

bitfield!(Leaf19Ecx, u32, {
    /// If 1, the NoBackup parameter to LOADIWKEY is supported.
    loadikey_no_backup: 0,
    key_source_encoding_one: 1,
    // 2..=31 reserved
});

bitfield!(Leaf19Edx, u32, {
    // reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 1A
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf1AEax, u32, {
    /// Native model ID of the core. The core-type and native model ID can be used to uniquely
    /// identify the microarchitecture of the core. This native model ID is not unique across core
    /// types, and not related to the model ID reported in CPUID leaf 01H, and does not identify the
    /// SOC.
    native_id: 0..24,
    /// Core type
    /// - 10H: Reserved
    /// - 20H: Intel Atom®
    /// - 30H: Reserved
    /// - 40H: Intel® Core™
    core_type: 24..32,
});

bitfield!(Leaf1AEbx, u32, {
    // Reserved.
});

bitfield!(Leaf1AEcx, u32, {
    // Reserved.
});

bitfield!(Leaf1AEdx, u32, {
    // Reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 1C
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf1CEax, u32, {
    /// Supported LBR Depth Values. For each bit n set in this field, the IA32_LBR_DEPTH.DEPTH value
    /// 8*(n+1) is supported.
    lbr_depth_values: 0..8,
    // 9..=28 reserved
    /// Deep C-state Reset. If set, indicates that LBRs may be cleared on an MWAIT that requests a
    /// C-state numerically greater than C1.
    deep_c_state: 30,
    /// IP Values Contain LIP. If set, LBR IP values contain LIP. If clear, IP values contain Effective IP.
    ip_values_contain_lip: 31,
});

bitfield!(Leaf1CEbx, u32, {
    /// CPL Filtering Supported. If set, the processor supports setting IA32_LBR_CTL[2:1] to
    /// non-zero value.
    cpl_filtering: 0,
    /// Branch Filtering Supported. If set, the processor supports setting IA32_LBR_CTL[22:16] to
    /// nonzero value.
    branch_filtering: 1,
    /// Call-stack Mode Supported. If set, the processor supports setting IA32_LBR_CTL[3] to 1.
    call_stack_mode: 2,
    // 3..=31 reserved
});

bitfield!(Leaf1CEcx, u32, {
    /// Mispredict Bit Supported. IA32_LBR_x_INFO[63] holds indication of branch misprediction
    /// (MISPRED).
    mispredict_bit: 0,
    /// Timed LBRs Supported. IA32_LBR_x_INFO[15:0] holds CPU cycles since last LBR entry (CYC_CNT),
    /// and IA32_LBR_x_INFO[60] holds an indication of whether the value held there is valid
    /// (CYC_CNT_VALID).
    timed_lbrs: 1,
    /// Branch Type Field Supported. IA32_LBR_INFO_x[59:56] holds indication of the recorded
    /// operation's branch type (BR_TYPE).
    branch_type_field: 2,
    // 3..=31 reserved
});

bitfield!(Leaf1CEdx, u32, {
    // 3..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 1F
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf1FEax, u32, {
    /// Number of bits to shift right on x2APIC ID to get a unique topology ID of the next level
    /// type*. All logical processors with the same next level ID share current level.
    right_shift_2xapic_id_unique_top_id: 0..5,
    // 5..=31 reserved
});

bitfield!(Leaf1FEbx, u32, {
    /// Number of logical processors at this level type. The number reflects configuration as
    /// shipped by Intel**.
    ///
    /// ** Software must not use EBX[15:0] to enumerate processor topology of the system. This value
    /// in this field (EBX[15:0]) is only intended for display/diagnostic purposes. The actual
    /// number of logical processors available to BIOS/OS/Applications may be different from the
    /// value of EBX[15:0], depending on software and platform hardware configurations.
    logical_processors: 0..16,
    // 16..=31 reserved
});

bitfield!(Leaf1FEcx, u32, {
    /// Level number. Same value in ECX input.
    level_number: 0..8,
    /// Level type***.
    ///
    /// *** The value of the “level type” field is not related to level numbers in any way, higher
    /// “level type” values do not mean higher levels. Level type field has the following encoding:
    ///
    /// - 0: Invalid.
    /// - 1: SMT.
    /// - 2: Core.
    /// - 3: Module.
    /// - 4: Tile.
    /// - 5: Die.
    /// - 6-255: Reserved.
    level_type: 8..16,
    // 16..=31 reserved
});

bitfield!(Leaf1FEdx, u32, {
    /// x2APIC ID the current logical processor.
    x2apic_id: 0..32,
});
// -------------------------------------------------------------------------------------------------
// Leaf 20
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf20Eax, u32, {
    /// Reports the maximum number of sub-leaves that are supported in leaf 20H.
    max_subleaves: 0..32,
});

bitfield!(Leaf20Ebx, u32, {
    // Indicates which bits may be set in the IA32_HRESET_ENABLE MSR to enable reset of different
    // components of hardware-maintained history.

    /// Indicates support for both HRESET’s EAX[0] parameter, and IA32_HRESET_ENABLE[0] set by the
    /// OS to enable reset of Intel® Thread Director history.
    hreset: 0,
    // 1..=31 reserved
});

bitfield!(Leaf20Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf20Edx, u32, {
    // Reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000000
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000000Eax, u32, {
    /// Maximum Input Value for Extended Function CPUID Information.
    max_extend_function_input: 0..32,
});

bitfield!(Leaf80000000Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf80000000Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf80000000Edx, u32, {
    // Reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000001
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000001Eax, u32, {
    /// Extended Processor Signature and Feature Bits.
    extended_processor_signature_and_feature_bits: 0..32,
});

bitfield!(Leaf80000001Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf80000001Ecx, u32, {
    /// LAHF/SAHF available in 64-bit mode.*
    ///
    /// * LAHF and SAHF are always available in other modes, regardless of the enumeration of this
    /// feature flag.
    lahf_sahf: 0,
    // 1..=4 reserved
    /// LZCNT.
    lzcnt: 5,
    // 6..=7 reserved
    /// PREFETCHW.
    prefetchcw: 8,
    // 9..=31 reserved
});

bitfield!(Leaf80000001Edx, u32, {
    // 0..=10 reserved
    /// SYSCALL/SYSRET.**
    ///
    /// ** Intel processors support SYSCALL and SYSRET only in 64-bit mode. This feature flag is
    /// always enumerated as 0 outside 64-bit mode.
    syscall_sysret: 11,
    // 12..=19 reserved
    /// Execute Disable Bit available.
    execute_disable_bit: 20,
    // 21..=25 reserved
    /// 1-GByte pages are available if 1.
    pages_1g: 26,
    /// RDTSCP and IA32_TSC_AUX are available if 1.
    rdtscp_and_ia32_tsc_aux: 27,
    // Reserved.
    /// Intel® 64 Architecture available if 1.
    arch64: 29,
    // 30..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000005
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000005Eax, u32, {
    // Reserved.
});

bitfield!(Leaf80000005Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf80000005Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf80000005Edx, u32, {
    // Reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000006
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000006Eax, u32, {
    // Reserved.
});

bitfield!(Leaf80000006Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf80000006Ecx, u32, {
    /// Cache Line size in bytes.
    cache_line_size: 0..8,
    // 8..=11 reserved
    /// L2 Associativity field *.
    ///
    /// * L2 associativity field encodings:
    /// - 00H - Disabled 08H - 16 ways
    /// - 01H - 1 way (direct mapped) 09H - Reserved
    /// - 02H - 2 ways 0AH - 32 ways
    /// - 03H - Reserved 0BH - 48 ways
    /// - 04H - 4 ways 0CH - 64 ways
    /// - 05H - Reserved 0DH - 96 ways
    /// - 06H - 8 ways 0EH - 128 ways
    /// - 07H - See CPUID leaf 04H, sub-leaf 2** 0FH - Fully associative
    ///
    /// ** CPUID leaf 04H provides details of deterministic cache parameters, including the L2 cache
    /// in sub-leaf 2
    l2_associativity: 12..16,
    /// Cache size in 1K units.
    cache_size_1k_units: 16..32,
});

bitfield!(Leaf80000006Edx, u32, {
    // Reserved.
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000007
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000007Eax, u32, {
    // Reserved.
});

bitfield!(Leaf80000007Ebx, u32, {
    // Reserved.
});

bitfield!(Leaf80000007Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf80000007Edx, u32, {
    // 0..=7 reserved
    invariant_tsc: 8,
    // 9..=31 reserved
});
// -------------------------------------------------------------------------------------------------
// Leaf 80000008
// -------------------------------------------------------------------------------------------------

bitfield!(Leaf80000008Eax, u32, {
    /// #Physical Address Bits*.
    ///
    /// * If CPUID.80000008H:EAX[7:0] is supported, the maximum physical address number supported
    /// should come from this field.
    physical_address_bits: 0..8,
    /// #Linear Address Bits.
    linear_address_bits: 8..16,
    // 8..=31 reserved
});

bitfield!(Leaf80000008Ebx, u32, {
    // 0..=8 reserved
    wbnoinvd: 9,
    // 10..=31 reserved

});

bitfield!(Leaf80000008Ecx, u32, {
    // Reserved.
});

bitfield!(Leaf80000008Edx, u32, {
    // Reserved.
});
