extern crate devices;
extern crate sys_util;
extern crate x86_64;

use std::result;

use kvm::*;
use sys_util::{EventFd, GuestAddress, GuestMemory};

use x86_64::{interrupts, regs};

pub const KVM_TSS_ADDRESS: usize = 0xfffbd000;
//x86_64 specific values
const KERNEL_64BIT_ENTRY_OFFSET: usize = 0x200;
const BOOT_STACK_POINTER: usize = 0x8000;

#[derive(Debug)]
pub enum Error {
    AlreadyRunning,
    VcpuCountOverflow,
    GuestMemory(sys_util::GuestMemoryError),
    Kvm(sys_util::Error),
    VmFd(sys_util::Error),
    VcpuFd(sys_util::Error),
    VmSetup(sys_util::Error),
    VcpuRun(sys_util::Error),
    GetSupportedCpusFailed(sys_util::Error),
    SetSupportedCpusFailed(sys_util::Error),
    NotEnoughMemory,
    NoMemoryEntry,
    LocalIntConfiguration(interrupts::Error),
    SetUserMemoryRegion(sys_util::Error),
    /// The kernel extends past the end of RAM
    KernelOffsetPastEnd,
    /// Error configuring the MSR registers
    MSRSConfiguration(regs::Error),
    /// Error configuring the general purpose registers
    REGSConfiguration(regs::Error),
    /// Error configuring the special registers
    SREGSConfiguration(regs::Error),
    /// Error configuring the floating point related registers
    FPUConfiguration(regs::Error),
    EventFd(sys_util::Error),
    Irq(sys_util::Error),
}
pub type Result<T> = result::Result<T, Error>;

impl ::std::convert::From<sys_util::Error> for Error {
    fn from(e: sys_util::Error) -> Error {
        Error::SetUserMemoryRegion(e)
    }
}

/// A wrapper around creating and using a VM.
pub struct Vm {
    fd: VmFd,
    guest_mem: Option<GuestMemory>,
}

impl Vm {
    /// Constructs a new `Vm` using the given `Kvm` instance.
    pub fn new(kvm: &Kvm) -> Result<Self> {
        //create fd for interacting with kvm-vm specific functions
        let vm_fd = VmFd::new(&kvm).map_err(Error::VmFd)?;

        Ok(Vm {
            fd: vm_fd,
            guest_mem: None,
        })
    }

    /// Currently this is x86 specific (because of the TSS address setup)
    pub fn memory_init(&mut self, guest_mem: GuestMemory) -> Result<()> {
        guest_mem.with_regions(|index, guest_addr, size, host_addr| {
            // Safe because the guest regions are guaranteed not to overlap.
            self.fd.set_user_memory_region(
                index as u32,
                guest_addr.offset() as u64,
                size as u64,
                host_addr as u64,
                0,
            )
        })?;
        self.guest_mem = Some(guest_mem);

        let tss_addr = GuestAddress(KVM_TSS_ADDRESS);
        self.fd
            .set_tss_address(tss_addr.offset())
            .map_err(Error::VmSetup)?;

        Ok(())
    }

    /// This function creates the irq chip and adds 2 interrupt events to the IRQ
    pub fn setup_irqchip(&self, com_evt_1_3: &EventFd, com_evt_2_4: &EventFd) -> Result<()> {
        self.fd.create_irq_chip().map_err(Error::VmSetup)?;

        self.fd.register_irqfd(com_evt_1_3, 4).map_err(Error::Irq)?;
        self.fd.register_irqfd(com_evt_2_4, 3).map_err(Error::Irq)?;

        Ok(())
    }

    pub fn create_pit(&self) -> Result<()> {
        self.fd.create_pit2().map_err(Error::VmSetup)?;
        Ok(())
    }

    /// Gets a reference to the guest memory owned by this VM.
    ///
    /// Note that `GuestMemory` does not include any device memory that may have been added after
    /// this VM was constructed.
    pub fn get_memory(&self) -> Option<&GuestMemory> {
        self.guest_mem.as_ref()
    }

    /// Gets a reference to the kvm file descriptor owned by this VM.
    ///
    pub fn get_fd(&self) -> &VmFd {
        &self.fd
    }
}

// constants for setting the fields of kvm_cpuid2 structures
// CPUID bits in ebx, ecx, and edx.
const EBX_CLFLUSH_CACHELINE: u32 = 8; // Flush a cache line size.
const EBX_CLFLUSH_SIZE_SHIFT: u32 = 8; // Bytes flushed when executing CLFLUSH.
const EBX_CPU_COUNT_SHIFT: u32 = 16; // The logical processor count .
const EBX_APICID_SHIFT: u32 = 24; // The (fixed) default APIC ID.
const ECX_EPB_SHIFT: u32 = 3; // "Energy Performance Bias" bit.
const ECX_TSC_DEADLINE_TIMER_SHIFT: u32 = 24;
const ECX_HYPERVISOR_SHIFT: u32 = 31; // Flag to be set when the cpu is running on a hypervisor.
const ECX_LEVEL_TYPE_SHIFT: u32 = 8; // Shift for setting level type for leaf 11
const EDX_HTT_SHIFT: u32 = 28; // Hyper Threading Enabled.

// Deterministic Cache Parameters Leaf
const EAX_CACHE_LEVEL: u32 = 5;
const EAX_MAX_ADDR_IDS_SHARING_CACHE: u32 = 14;
const EAX_MAX_ADDR_IDS_IN_PACKAGE: u32 = 26;

const LEAFBH_LEVEL_TYPE_INVALID: u32 = 0;
const LEAFBH_LEVEL_TYPE_THREAD: u32 = 1;
const LEAFBH_LEVEL_TYPE_CORE: u32 = 2;

// The APIC ID shift in leaf 0xBh specifies the number of bits to shit the x2APIC ID to get a
// unique topology of the next level. This allows 64 logical processors/package.
const LEAFBH_INDEX1_APICID_SHIFT: u32 = 6;

/// A wrapper around creating and using a kvm-based VCPU
pub struct Vcpu {
    cpuid: CpuId,
    fd: VcpuFd,
    id: u8,
}

impl Vcpu {
    /// Constructs a new VCPU for `vm`.
    ///
    /// The `id` argument is the CPU number between [0, max vcpus).
    pub fn new(id: u8, vm: &Vm) -> Result<Self> {
        let kvm_vcpu = VcpuFd::new(id, &vm.fd).map_err(Error::VcpuFd)?;
        // Initially the cpuid per vCPU is the one supported by this VM
        Ok(Vcpu {
            fd: kvm_vcpu,
            cpuid: vm.fd.get_supported_cpuid(),
            id,
        })
    }

    /// This function is used for setting leaf 01H EBX[23-16]
    /// The maximum number of addressable logical CPUs is computed as the closest power of 2
    /// higher or equal to the CPU count configured by the user
    fn get_max_addressable_lprocessors(cpu_count: u8) -> result::Result<u8, Error> {
        let mut max_addressable_lcpu = (cpu_count as f64).log2().ceil();
        max_addressable_lcpu = (2 as f64).powf(max_addressable_lcpu);
        // check that this number is still an u8
        if max_addressable_lcpu > u8::max_value().into() {
            return Err(Error::VcpuCountOverflow);
        }
        Ok(max_addressable_lcpu as u8)
    }

    /// Sets up the cpuid entries for the given vcpu
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn filter_cpuid(&mut self, cpu_count: u8) -> Result<()> {
        let entries = self.cpuid.mut_entries_slice();
        let max_addr_cpu = Vcpu::get_max_addressable_lprocessors(cpu_count).unwrap() as u32;
        for entry in entries.iter_mut() {
            match entry.function {
                1 => {
                    // X86 hypervisor feature
                    if entry.index == 0 {
                        entry.ecx |= 1 << ECX_TSC_DEADLINE_TIMER_SHIFT;
                        entry.ecx |= 1 << ECX_HYPERVISOR_SHIFT;
                    }
                    entry.ebx = ((self.id as u32) << EBX_APICID_SHIFT) as u32
                        | (EBX_CLFLUSH_CACHELINE << EBX_CLFLUSH_SIZE_SHIFT);
                    entry.ebx |= max_addr_cpu << EBX_CPU_COUNT_SHIFT;
                    // Make sure that Hyperthreading is disabled
                    entry.edx &= !(1 << EDX_HTT_SHIFT);
                    // Enable Hyperthreading for even vCPU count so you don't end up with
                    // an even and > 1 number of sibilings
                    if cpu_count > 1 && cpu_count % 2 == 0 {
                        entry.edx |= 1 << EDX_HTT_SHIFT;
                    }
                }
                4 => {
                    // Deterministic Cache Parameters Leaf
                    // Only use the last 3 bits of EAX[5:32] because the level is encoded in EAX[5:7]
                    let cache_level = (entry.eax >> EAX_CACHE_LEVEL) & (0b111 as u32);
                    match cache_level {
                        // L1 & L2 Cache
                        1 | 2 => {
                            // Set the maximum addressable IDS sharing the data cache to zero
                            // when you only have 1 vcpu because there are no other threads on
                            // the machine to share the data/instruction cache
                            // This sets EAX[25:14]
                            entry.eax &= !(0b111111111111 << EAX_MAX_ADDR_IDS_SHARING_CACHE);
                            if cpu_count > 1 {
                                // Hyperthreading is enabled by default for vcpu_count > 2
                                entry.eax |= 1 << EAX_MAX_ADDR_IDS_SHARING_CACHE;
                            }
                        }
                        // L3 Cache
                        3 => {
                            // Set the maximum addressable IDS sharing the data cache to zero
                            // when you only have 1 vcpu because there are no other threads on
                            // the machine to share the data/instruction cache
                            // This sets EAX[25:14]
                            entry.eax &= !(0b111111111111 << EAX_MAX_ADDR_IDS_SHARING_CACHE);
                            if cpu_count > 1 {
                                entry.eax |=
                                    ((cpu_count - 1) as u32) << EAX_MAX_ADDR_IDS_SHARING_CACHE;
                            }
                        }
                        _ => (),
                    }

                    // Maximum number of addressable IDs for processor cores in the physical package
                    // should be the same on all cache levels
                    // set this to 0 because there is only 1 core available for vcpu_count <= 2
                    // This sets EAX[31:26]
                    entry.eax &= !(0b111111 << EAX_MAX_ADDR_IDS_IN_PACKAGE);
                    if cpu_count > 2 {
                        // we have HT enabled by default, so we will have cpu_count/2 cores in package
                        entry.eax |= (((cpu_count >> 1) - 1) as u32) << EAX_MAX_ADDR_IDS_IN_PACKAGE;
                    }
                }
                6 => {
                    // Clear X86 EPB feature.  No frequency selection in the hypervisor.
                    entry.ecx &= !(1 << ECX_EPB_SHIFT);
                }
                11 => {
                    // Hide the actual topology of the underlying host
                    match entry.index {
                        0 => {
                            // Thread Level Topology; index = 0
                            if cpu_count == 1 {
                                // No APIC ID at the next level, set EAX to 0
                                entry.eax = 0;
                                // Set the numbers of logical processors to 1
                                entry.ebx = 1;
                                // There are no hyperthreads for 1 VCPU, set the level type = 2 (Core)
                                entry.ecx = LEAFBH_LEVEL_TYPE_CORE << ECX_LEVEL_TYPE_SHIFT;
                            } else {
                                // To get the next level APIC ID, shift right with 1 because we have
                                // maximum 2 hyperthreads per core that can be represented with 1 bit
                                entry.eax = 1;
                                // 2 logical cores at this level
                                entry.ebx = 2;
                                // enforce this level to be of type thread
                                entry.ecx = LEAFBH_LEVEL_TYPE_THREAD << ECX_LEVEL_TYPE_SHIFT;
                            }
                        }
                        1 => {
                            // Core Level Processor Topology; index = 1
                            entry.eax = LEAFBH_INDEX1_APICID_SHIFT;
                            if cpu_count == 1 {
                                // For 1 vCPU, this level is invalid
                                entry.ebx = 0;
                                // ECX[7:0] = entry.index; ECX[15:8] = 0 (Invalid Level)
                                entry.ecx = (entry.index as u32)
                                    | (LEAFBH_LEVEL_TYPE_INVALID << ECX_LEVEL_TYPE_SHIFT);
                            } else {
                                entry.ebx = cpu_count as u32;
                                entry.ecx = (entry.index as u32)
                                    | (LEAFBH_LEVEL_TYPE_CORE << ECX_LEVEL_TYPE_SHIFT);
                            }
                        }
                        level => {
                            // Core Level Processor Topology; index >=2
                            // No other levels available; This should already be set to correctly,
                            // and it is added here as a "re-enforcement" in case we run on
                            // different hardware
                            entry.eax = 0;
                            entry.ebx = 0;
                            entry.ecx = level;
                        }
                    }
                    // EDX bits 31..0 contain x2APIC ID of current logical processor
                    // x2APIC increases the size of the APIC ID from 8 bits to 32 bits
                    entry.edx = self.id as u32;
                }
                _ => (),
            }
        }

        Ok(())
    }

    /// Returns a clone of the CPUID entries of this vCPU
    /// For now this function is only used for testing; the cfg(test) should be removed when
    /// this function will be used for configuring the cpu features
    #[cfg(test)]
    pub fn get_cpuid(&self) -> CpuId {
        return self.cpuid.clone();
    }

    /// /// Configures the vcpu and should be called once per vcpu from the vcpu's thread.
    ///
    /// # Arguments
    ///
    /// * `kernel_load_offset` - Offset from `guest_mem` at which the kernel starts.
    /// nr cpus is required for checking populating the kvm_cpuid2 entry for ebx and edx registers
    pub fn configure(
        &mut self,
        nrcpus: u8,
        kernel_start_addr: GuestAddress,
        vm: &Vm,
    ) -> Result<()> {
        self.filter_cpuid(nrcpus)?;

        self.fd
            .set_cpuid2(&self.cpuid)
            .map_err(Error::SetSupportedCpusFailed)?;

        regs::setup_msrs(&self.fd).map_err(Error::MSRSConfiguration)?;
        // Safe to unwrap because this method is called after the VM is configured
        let vm_memory = vm.get_memory().unwrap();
        let kernel_end = vm_memory
            .checked_offset(kernel_start_addr, KERNEL_64BIT_ENTRY_OFFSET)
            .ok_or(Error::KernelOffsetPastEnd)?;
        regs::setup_regs(
            &self.fd,
            (kernel_end).offset() as u64,
            BOOT_STACK_POINTER as u64,
            x86_64::ZERO_PAGE_OFFSET as u64,
        ).map_err(Error::REGSConfiguration)?;
        regs::setup_fpu(&self.fd).map_err(Error::FPUConfiguration)?;
        regs::setup_sregs(vm_memory, &self.fd).map_err(Error::SREGSConfiguration)?;
        interrupts::set_lint(&self.fd).map_err(Error::LocalIntConfiguration)?;
        Ok(())
    }

    /// Runs the VCPU until it exits, returning the reason.
    ///
    /// Note that the state of the VCPU and associated VM must be setup first for this to do
    /// anything useful.
    pub fn run(&self) -> Result<VcpuExit> {
        match self.fd.run() {
            Ok(v) => Ok(v),
            Err(e) => return Err(Error::VcpuRun(<sys_util::Error>::new(e.errno()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kvm_sys::bindings::kvm_cpuid_entry2;

    #[test]
    fn create_vm() {
        let kvm = Kvm::new().unwrap();
        let gm = GuestMemory::new(&vec![(GuestAddress(0), 0x10000)]).unwrap();
        let mut vm = Vm::new(&kvm).expect("new vm failed");
        assert!(vm.memory_init(gm).is_ok());
    }

    #[test]
    fn get_memory() {
        let kvm = Kvm::new().unwrap();
        let gm = GuestMemory::new(&vec![(GuestAddress(0), 0x1000)]).unwrap();
        let mut vm = Vm::new(&kvm).expect("new vm failed");
        assert!(vm.memory_init(gm).is_ok());
        let obj_addr = GuestAddress(0xf0);
        vm.get_memory()
            .unwrap()
            .write_obj_at_addr(67u8, obj_addr)
            .unwrap();
        let read_val: u8 = vm.get_memory()
            .unwrap()
            .read_obj_from_addr(obj_addr)
            .unwrap();
        assert_eq!(read_val, 67u8);
    }

    #[test]
    fn create_vcpu() {
        let kvm = Kvm::new().unwrap();
        let gm = GuestMemory::new(&vec![(GuestAddress(0), 0x10000)]).unwrap();
        let mut vm = Vm::new(&kvm).expect("new vm failed");
        assert!(vm.memory_init(gm).is_ok());
        Vcpu::new(0, &mut vm).unwrap();
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_get_max_addressable_lprocessors() {
        assert_eq!(Vcpu::get_max_addressable_lprocessors(1).unwrap(), 1);
        assert_eq!(Vcpu::get_max_addressable_lprocessors(2).unwrap(), 2);
        assert_eq!(Vcpu::get_max_addressable_lprocessors(4).unwrap(), 4);
        assert_eq!(Vcpu::get_max_addressable_lprocessors(6).unwrap(), 8);
        assert!(Vcpu::get_max_addressable_lprocessors(u8::max_value()).is_err());
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_cpuid() {
        let kvm = Kvm::new().unwrap();
        let mut vm = Vm::new(&kvm).unwrap();
        let mut vcpu = Vcpu::new(0, &mut vm).unwrap();
        assert_eq!(vcpu.get_cpuid(), vm.fd.get_supported_cpuid());
        assert!(vcpu.filter_cpuid(1).is_ok());
        assert!(vcpu.fd.set_cpuid2(&vcpu.cpuid).is_ok());
        let entries = vcpu.cpuid.mut_entries_slice();
        // TODO: add tests for the other cpuid leaves
        // Test the extended topology
        // See https://www.scss.tcd.ie/~jones/CS4021/processor-identification-cpuid-instruction-note.pdf
        let leaf11_index0 = kvm_cpuid_entry2 {
            function: 11,
            index: 0,
            flags: 1,
            eax: 0,
            ebx: 1,                                              // nr of hyperthreads/core
            ecx: LEAFBH_LEVEL_TYPE_CORE << ECX_LEVEL_TYPE_SHIFT, // ECX[15:8] = 2 (Core Level)
            edx: 0,                                              // EDX = APIC ID = 0
            padding: [0, 0, 0],
        };
        assert!(entries.contains(&leaf11_index0));
        let leaf11_index1 = kvm_cpuid_entry2 {
            function: 11,
            index: 1,
            flags: 1,
            eax: LEAFBH_INDEX1_APICID_SHIFT,
            ebx: 0,
            ecx: 1, // ECX[15:8] = 0 (Invalid Level) & ECX[7:0] = 1 (Level Number)
            edx: 0, // EDX = APIC ID = 0
            padding: [0, 0, 0],
        };
        assert!(entries.contains(&leaf11_index1));
        let leaf11_index2 = kvm_cpuid_entry2 {
            function: 11,
            index: 2,
            flags: 1,
            eax: 0,
            ebx: 0, // nr of hyperthreads/core
            ecx: 2, // ECX[15:8] = 0 (Invalid Level) & ECX[7:0] = 2 (Level Number)
            edx: 0, // EDX = APIC ID = 0
            padding: [0, 0, 0],
        };
        assert!(entries.contains(&leaf11_index2));
    }

    #[test]
    fn run_code() {
        use std::io::{self, Write};
        // This example based on https://lwn.net/Articles/658511/
        let code = [
            0xba, 0xf8, 0x03 /* mov $0x3f8, %dx */, 0x00, 0xd8 /* add %bl, %al */, 0x04,
            '0' as u8 /* add $'0', %al */, 0xee /* out %al, (%dx) */, 0xb0,
            '\n' as u8 /* mov $'\n', %al */, 0xee /* out %al, (%dx) */,
            0xf4 /* hlt */,
        ];

        let mem_size = 0x1000;
        let load_addr = GuestAddress(0x1000);
        let mem = GuestMemory::new(&vec![(load_addr, mem_size)]).unwrap();

        let kvm = Kvm::new().expect("new kvm failed");
        let mut vm = Vm::new(&kvm).expect("new vm failed");
        assert!(vm.memory_init(mem).is_ok());
        vm.get_memory()
            .unwrap()
            .write_slice_at_addr(&code, load_addr)
            .expect("Writing code to memory failed.");

        let vcpu = Vcpu::new(0, &mut vm).expect("new vcpu failed");

        let mut vcpu_sregs = vcpu.fd.get_sregs().expect("get sregs failed");
        assert_ne!(vcpu_sregs.cs.base, 0);
        assert_ne!(vcpu_sregs.cs.selector, 0);
        vcpu_sregs.cs.base = 0;
        vcpu_sregs.cs.selector = 0;
        vcpu.fd.set_sregs(&vcpu_sregs).expect("set sregs failed");

        let mut vcpu_regs = vcpu.fd.get_regs().expect("get regs failed");
        vcpu_regs.rip = 0x1000;
        vcpu_regs.rax = 2;
        vcpu_regs.rbx = 3;
        vcpu_regs.rflags = 2;
        vcpu.fd.set_regs(&vcpu_regs).expect("set regs failed");

        loop {
            match vcpu.run().expect("run failed") {
                VcpuExit::IoOut(0x3f8, data) => {
                    assert_eq!(data.len(), 1);
                    io::stdout().write(data).unwrap();
                }
                VcpuExit::Hlt => {
                    io::stdout().write(b"KVM_EXIT_HLT\n").unwrap();
                    break;
                }
                r => panic!("unexpected exit reason: {:?}", r),
            }
        }
    }
}
