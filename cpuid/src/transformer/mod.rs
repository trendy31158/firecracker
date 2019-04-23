// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

pub mod amd;
pub mod common;
pub mod intel;

pub use kvm_bindings::kvm_cpuid_entry2;
use kvm_ioctls::CpuId;

use brand_string::BrandString;
use brand_string::Reg as BsReg;

pub struct VmSpec {
    pub cpu_id: u8,
    pub cpu_count: u8,
    pub ht_enabled: bool,
    brand_string: BrandString,
}

impl VmSpec {
    pub fn new(vendor_id: &[u8; 12], cpu_id: u8, cpu_count: u8, ht_enabled: bool) -> VmSpec {
        VmSpec {
            cpu_id,
            cpu_count,
            ht_enabled,
            brand_string: BrandString::from_vendor_id(vendor_id),
        }
    }

    pub fn brand_string(&self) -> &BrandString {
        &self.brand_string
    }
}

/// Errors associated with processing the CPUID leaves.
#[derive(Debug, Clone)]
pub enum Error {
    /// The maximum number of addressable logical CPUs cannot be stored in an `u8`.
    VcpuCountOverflow,
    /// The max size has been exceeded
    SizeLimitExceeded,
    /// A call to an internal helper method failed
    InternalError(super::common::Error),
}

pub type EntryTransformerFn =
    fn(entry: &mut kvm_cpuid_entry2, vm_spec: &VmSpec) -> Result<(), Error>;

/// Generic trait that provides methods for transforming the cpuid
///
pub trait CpuidTransformer {
    /// Trait main function. It processes the cpuid and makes the desired transformations.
    /// The default logic can be overwritten if needed. For example see `AmdCpuidTransformer`.
    ///
    fn process_cpuid(&self, cpuid: &mut CpuId, vm_spec: &VmSpec) -> Result<(), Error> {
        self.process_entries(cpuid, vm_spec)
    }

    /// Iterates through all the cpuid entries and calls the associated transformer for each one.
    ///
    fn process_entries(&self, cpuid: &mut CpuId, vm_spec: &VmSpec) -> Result<(), Error> {
        for entry in cpuid.mut_entries_slice().iter_mut() {
            let maybe_transformer_fn = self.entry_transformer_fn(entry);

            if let Some(transformer_fn) = maybe_transformer_fn {
                transformer_fn(entry, vm_spec)?;
            }
        }

        Ok(())
    }

    /// Gets the associated transformer for a cpuid entry
    ///
    fn entry_transformer_fn(&self, _entry: &mut kvm_cpuid_entry2) -> Option<EntryTransformerFn> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common::get_vendor_id;
    use kvm_bindings::kvm_cpuid_entry2;

    const PROCESSED_FN: u32 = 1;
    const EXPECTED_INDEX: u32 = 100;

    fn transform_entry(entry: &mut kvm_cpuid_entry2, _vm_spec: &VmSpec) -> Result<(), Error> {
        entry.index = EXPECTED_INDEX;

        Ok(())
    }

    struct MockCpuidTransformer {}

    impl CpuidTransformer for MockCpuidTransformer {
        fn entry_transformer_fn(&self, entry: &mut kvm_cpuid_entry2) -> Option<EntryTransformerFn> {
            match entry.function {
                PROCESSED_FN => Some(transform_entry),
                _ => None,
            }
        }
    }

    #[test]
    fn test_process_cpuid() {
        let num_entries = 5;

        let mut cpuid = CpuId::new(num_entries);
        let vm_spec = VmSpec::new(&get_vendor_id().unwrap(), 0, 1, false);
        cpuid.mut_entries_slice()[0].function = PROCESSED_FN;
        assert!(MockCpuidTransformer {}
            .process_cpuid(&mut cpuid, &vm_spec)
            .is_ok());

        assert!(cpuid.mut_entries_slice().len() == num_entries);
        for entry in cpuid.mut_entries_slice().iter() {
            match entry.function {
                PROCESSED_FN => {
                    assert_eq!(entry.index, EXPECTED_INDEX);
                }
                _ => {
                    assert_ne!(entry.index, EXPECTED_INDEX);
                }
            }
        }
    }
}
