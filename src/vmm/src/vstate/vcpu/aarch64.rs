// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

use std::result;

use kvm_ioctls::*;
use logger::{error, IncMetric, METRICS};
use versionize::{VersionMap, Versionize, VersionizeResult};
use versionize_derive::Versionize;
use vm_memory::{Address, GuestAddress, GuestMemoryMmap};

use crate::arch::aarch64::regs::Aarch64Register;
use crate::vcpu::VcpuConfig;
use crate::vstate::vcpu::VcpuEmulation;
use crate::vstate::vm::Vm;

/// Errors associated with the wrappers over KVM ioctls.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error configuring the vcpu registers: {0}")]
    ConfigureRegisters(crate::arch::aarch64::regs::Error),
    #[error("Error creating vcpu: {0}")]
    CreateVcpu(kvm_ioctls::Error),
    #[error("Error getting the vcpu preferred target: {0}")]
    GetPreferredTarget(kvm_ioctls::Error),
    #[error("Error initializing the vcpu: {0}")]
    Init(kvm_ioctls::Error),
    #[error("Error applying template to the vcpu: {0}")]
    ApplyCpuTemplate(crate::arch::aarch64::regs::Error),
    #[error("Failed to restore the state of the vcpu: {0}")]
    RestoreState(crate::arch::aarch64::regs::Error),
    #[error("Failed to save the state of the vcpu: {0}")]
    SaveState(crate::arch::aarch64::regs::Error),
}

type Result<T> = result::Result<T, Error>;

/// A wrapper around creating and using a kvm aarch64 vcpu.
pub struct KvmVcpu {
    pub index: u8,
    pub fd: VcpuFd,

    pub mmio_bus: Option<devices::Bus>,

    mpidr: u64,
}
pub type KvmVcpuConfigureError = Error;
impl KvmVcpu {
    /// Constructs a new kvm vcpu with arch specific functionality.
    ///
    /// # Arguments
    ///
    /// * `index` - Represents the 0-based CPU index between [0, max vcpus).
    /// * `vm` - The vm to which this vcpu will get attached.
    pub fn new(index: u8, vm: &Vm) -> Result<Self> {
        let kvm_vcpu = vm.fd().create_vcpu(index.into()).map_err(Error::CreateFd)?;

        Ok(KvmVcpu {
            index,
            fd: kvm_vcpu,
            mmio_bus: None,
            mpidr: 0,
        })
    }

    /// Gets the MPIDR register value.
    pub fn get_mpidr(&self) -> u64 {
        self.mpidr
    }

    /// Configures an aarch64 specific vcpu for booting Linux.
    ///
    /// # Arguments
    ///
    /// * `guest_mem` - The guest memory used by this microvm.
    /// * `kernel_load_addr` - Offset from `guest_mem` at which the kernel is loaded.
    pub fn configure(
        &mut self,
        guest_mem: &GuestMemoryMmap,
        kernel_load_addr: GuestAddress,
        _vcpu_config: &VcpuConfig,
    ) -> std::result::Result<(), KvmVcpuConfigureError> {
        // TODO - Apply CPU config
        crate::arch::aarch64::regs::setup_boot_regs(
            &self.fd,
            self.index,
            kernel_load_addr.raw_value(),
            guest_mem,
        )
        .map_err(Error::ConfigureRegisters)?;

        self.mpidr =
            crate::arch::aarch64::regs::read_mpidr(&self.fd).map_err(Error::ConfigureRegisters)?;

        Ok(())
    }

    /// Initializes an aarch64 specific vcpu for booting Linux.
    ///
    /// # Arguments
    ///
    /// * `vm_fd` - The kvm `VmFd` for this microvm.
    pub fn init(&self, vm_fd: &VmFd) -> Result<()> {
        let mut kvi: kvm_bindings::kvm_vcpu_init = kvm_bindings::kvm_vcpu_init::default();

        // This reads back the kernel's preferred target type.
        vm_fd
            .get_preferred_target(&mut kvi)
            .map_err(Error::GetPreferredTarget)?;
        // We already checked that the capability is supported.
        kvi.features[0] |= 1 << kvm_bindings::KVM_ARM_VCPU_PSCI_0_2;
        // Non-boot cpus are powered off initially.
        if self.index > 0 {
            kvi.features[0] |= 1 << kvm_bindings::KVM_ARM_VCPU_POWER_OFF;
        }
        self.fd.vcpu_init(&kvi).map_err(Error::Init)
    }

    /// Save the KVM internal state.
    pub fn save_state(&self) -> Result<VcpuState> {
        let mut state = VcpuState {
            mp_state: crate::arch::regs::get_mpstate(&self.fd).map_err(Error::SaveState)?,
            ..Default::default()
        };

        crate::arch::regs::save_core_registers(&self.fd, &mut state.regs)
            .map_err(Error::SaveState)?;

        crate::arch::regs::save_system_registers(&self.fd, &mut state.regs)
            .map_err(Error::SaveState)?;

        state.mpidr = crate::arch::aarch64::regs::read_mpidr(&self.fd).map_err(Error::SaveState)?;

        Ok(state)
    }

    /// Use provided state to populate KVM internal state.
    pub fn restore_state(&self, state: &VcpuState) -> Result<()> {
        crate::arch::regs::restore_registers(&self.fd, &state.regs).map_err(Error::RestoreState)?;

        crate::arch::regs::set_mpstate(&self.fd, state.mp_state).map_err(Error::RestoreState)?;

        Ok(())
    }

    /// Runs the vCPU in KVM context and handles the kvm exit reason.
    ///
    /// Returns error or enum specifying whether emulation was handled or interrupted.
    pub fn run_arch_emulation(&self, exit: VcpuExit) -> super::Result<VcpuEmulation> {
        METRICS.vcpu.failures.inc();
        // TODO: Are we sure we want to finish running a vcpu upon
        // receiving a vm exit that is not necessarily an error?
        error!("Unexpected exit reason on vcpu run: {:?}", exit);
        Err(super::Error::UnhandledKvmExit(format!("{:?}", exit)))
    }
}

/// Structure holding VCPU kvm state.
#[derive(Clone, Default, Versionize)]
pub struct VcpuState {
    pub mp_state: kvm_bindings::kvm_mp_state,
    pub regs: Vec<Aarch64Register>,
    // We will be using the mpidr for passing it to the VmState.
    // The VmState will give this away for saving restoring the icc and redistributor
    // registers.
    pub mpidr: u64,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::undocumented_unsafe_blocks)]
    use std::os::unix::io::AsRawFd;

    use vm_memory::GuestMemoryMmap;

    use super::*;
    use crate::vcpu::VcpuConfig;
    use crate::vstate::vm::tests::setup_vm;
    use crate::vstate::vm::Vm;

    fn setup_vcpu(mem_size: usize) -> (Vm, KvmVcpu, GuestMemoryMmap) {
        let (mut vm, vm_mem) = setup_vm(mem_size);
        let vcpu = KvmVcpu::new(0, &vm).unwrap();
        vcpu.init(vm.fd()).unwrap();
        vm.setup_irqchip(1).unwrap();

        (vm, vcpu, vm_mem)
    }

    fn init_vcpu(vcpu: &VcpuFd, vm: &VmFd) {
        let mut kvi: kvm_bindings::kvm_vcpu_init = kvm_bindings::kvm_vcpu_init::default();
        vm.get_preferred_target(&mut kvi).unwrap();
        vcpu.vcpu_init(&kvi).unwrap();
    }

    #[test]
    fn test_create_vcpu() {
        let (vm, _) = setup_vm(0x1000);

        unsafe { libc::close(vm.fd().as_raw_fd()) };

        let err = KvmVcpu::new(0, &vm);
        assert!(err.is_err());
        assert_eq!(
            err.err().unwrap().to_string(),
            "Error in opening the VCPU file descriptor: Bad file descriptor (os error 9)"
                .to_string()
        );
    }

    #[test]
    fn test_configure_vcpu() {
        let (_vm, mut vcpu, vm_mem) = setup_vcpu(0x10000);

        let vcpu_config = VcpuConfig {
            vcpu_count: 1,
            smt: false,
            cpu_config: Default::default(),
        };
        assert!(vcpu
            .configure(
                &vm_mem,
                GuestAddress(crate::arch::get_kernel_start()),
                &vcpu_config,
            )
            .is_ok());

        unsafe { libc::close(vcpu.fd.as_raw_fd()) };

        let err = vcpu.configure(
            &vm_mem,
            GuestAddress(crate::arch::get_kernel_start()),
            &vcpu_config,
        );
        assert!(err.is_err());
        assert_eq!(
            err.err().unwrap().to_string(),
            "Error configuring the general purpose registers: Failed to set processor state \
             register: Bad file descriptor (os error 9)"
                .to_string()
        );

        let (_vm, mut vcpu, vm_mem) = setup_vcpu(0x10000);
        unsafe { libc::close(vcpu.fd.as_raw_fd()) };
        let err = vcpu.configure(
            &vm_mem,
            GuestAddress(crate::arch::get_kernel_start()),
            &vcpu_config,
        );
        assert!(err.is_err());
        assert_eq!(
            err.err().unwrap().to_string(),
            "Error configuring the general purpose registers: Failed to set processor state \
             register: Bad file descriptor (os error 9)"
                .to_string()
        );
    }

    #[test]
    fn test_faulty_init_vcpu() {
        let (vm, vcpu, _) = setup_vcpu(0x10000);
        unsafe { libc::close(vm.fd().as_raw_fd()) };
        let err = vcpu.init(vm.fd());
        assert!(err.is_err());
        assert_eq!(
            err.err().unwrap().to_string(),
            "Error retrieving the vcpu preferred target: Bad file descriptor (os error 9)"
                .to_string()
        );
    }

    #[test]
    fn test_vcpu_save_restore_state() {
        let (mut vm, _vm_mem) = setup_vm(0x1000);
        let vcpu = KvmVcpu::new(0, &vm).unwrap();
        vm.setup_irqchip(1).unwrap();

        // Calling KVM_GET_REGLIST before KVM_VCPU_INIT will result in error.
        let res = vcpu.save_state();
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().to_string(),
            "Failed to save the state of the vcpu: Failed to get X0 register: Exec format error \
             (os error 8)"
                .to_string()
        );

        // Try to restore the register using a faulty state.
        let faulty_vcpu_state = VcpuState {
            regs: vec![Aarch64Register { id: 0, value: 0 }],
            ..Default::default()
        };

        let res = vcpu.restore_state(&faulty_vcpu_state);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().to_string(),
            "Failed to restore the state of the vcpu: Failed to set register: Exec format error \
             (os error 8)"
                .to_string()
        );

        init_vcpu(&vcpu.fd, vm.fd());
        let state = vcpu.save_state().expect("Cannot save state of vcpu");
        assert!(!state.regs.is_empty());
        vcpu.restore_state(&state)
            .expect("Cannot restore state of vcpu");
        let value = vcpu
            .fd
            .get_one_reg(0x6030_0000_0010_003E)
            .expect("Cannot get sp core register");
        assert!(state.regs.contains(&Aarch64Register {
            id: 0x6030_0000_0010_003E,
            value
        }));
    }

    #[test]
    fn test_setup_non_boot_vcpu() {
        let (vm, _) = setup_vm(0x1000);
        let vcpu1 = KvmVcpu::new(0, &vm).unwrap();
        assert!(vcpu1.init(vm.fd()).is_ok());
        let vcpu2 = KvmVcpu::new(1, &vm).unwrap();
        assert!(vcpu2.init(vm.fd()).is_ok());
    }
}
