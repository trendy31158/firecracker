use cpuid::cpu_leaf::*;
use kvm_sys::kvm_cpuid_entry2;

pub fn set_cpuid_entries(entries: &mut [kvm_cpuid_entry2]) {
    for entry in entries.iter_mut() {
        match entry.function {
            0x1 => {
                // Set CPU Basic Information
                // EAX[20:27] Extended Family ID = 0
                entry.eax &= !(0b11111111 << leaf_0x1::eax::EXTENDED_FAMILY_ID_SHIFT);

                // EAX[19:16] Extended Processor Model ID = 3 (Haswell)
                entry.eax &= !(0b1111 << leaf_0x1::eax::EXTENDED_PROCESSOR_MODEL_SHIFT);
                entry.eax |= 3 << leaf_0x1::eax::EXTENDED_PROCESSOR_MODEL_SHIFT;

                // EAX[13:12] Processor Type = 0 (Primary processor)
                entry.eax &= !(0b11 << leaf_0x1::eax::PROCESSOR_TYPE_SHIFT);

                // EAX[11:8] Processor Family = 6
                entry.eax &= !(0b1111 << leaf_0x1::eax::PROCESSOR_FAMILY_SHIFT);
                entry.eax |= 6 << leaf_0x1::eax::PROCESSOR_FAMILY_SHIFT;

                // EAX[7:4] Processor Model = 14
                entry.eax &= !(0b1111 << leaf_0x1::eax::PROCESSOR_MODEL_SHIFT);
                entry.eax |= 14 << leaf_0x1::eax::PROCESSOR_MODEL_SHIFT;

                // EAX[0:3] Stepping = 4
                entry.eax &= !(0b1111 as u32);
                entry.eax |= 4 as u32;

                // Disable Features
                entry.ecx &= !(1 << leaf_0x1::ecx::DTES64_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::MONITOR_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::DS_CPL_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::TM2_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::CNXT_ID);
                entry.ecx &= !(1 << leaf_0x1::ecx::SDBG_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::FMA_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::XTPR_UPDATE_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::PDCM_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::MOVBE_SHIFT);
                entry.ecx &= !(1 << leaf_0x1::ecx::OSXSAVE_SHIFT);

                entry.edx &= !(1 << leaf_0x1::edx::PSN_SHIFT);
                entry.edx &= !(1 << leaf_0x1::edx::DS_SHIFT);
                entry.edx &= !(1 << leaf_0x1::edx::ACPI_SHIFT);
                entry.edx &= !(1 << leaf_0x1::edx::SS_SHIFT);
                entry.edx &= !(1 << leaf_0x1::edx::TM_SHIFT);
                entry.edx &= !(1 << leaf_0x1::edx::PBE_SHIFT);
            }
            0x7 => {
                if entry.index == 0 {
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::SGX_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::BMI1_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::HLE_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::AVX2_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::FPDP_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::BMI2_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::INVPCID_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::BMI1_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::RTM_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::RDT_M_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::RDT_A_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::AVX512F_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::RDSEED_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::ADX_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::PT_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::AVX512CD_SHIFT);
                    entry.ebx &= !(1 << leaf_0x7::index0::ebx::SHA_SHIFT);

                    entry.ecx &= !(1 << leaf_0x7::index0::ecx::RDPID_SHIFT);
                    entry.ecx &= !(1 << leaf_0x7::index0::ecx::SGX_LC_SHIFT);
                }
            }
            0x80000001 => {
                entry.ecx &= !(1 << leaf_0x80000001::ecx::PREFETCH_SHIFT);
                entry.ecx &= !(1 << leaf_0x80000001::ecx::LZCNT_SHIFT);
                entry.edx &= !(1 << leaf_0x80000001::edx::PDPE1GB_SHIFT);
            }

            _ => (),
        }
    }
}
