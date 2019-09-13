// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

use std::mem;
use std::result;
use std::slice;
use std::{fmt, io};

use libc::c_char;

use arch_gen::x86::mpspec;
use memory_model::{DataInit, GuestAddress, GuestMemory};
use std::fmt::Formatter;

// This is a workaround to the Rust enforcement specifying that any implementation of a foreign
// trait (in this case `DataInit`) where:
// *    the type that is implementing the trait is foreign or
// *    all of the parameters being passed to the trait (if there are any) are also foreign
// is prohibited.
#[derive(Copy, Clone)]
struct MpcBusWrapper(mpspec::mpc_bus);
#[derive(Copy, Clone)]
struct MpcCpuWrapper(mpspec::mpc_cpu);
#[derive(Copy, Clone)]
struct MpcIntsrcWrapper(mpspec::mpc_intsrc);
#[derive(Copy, Clone)]
struct MpcIoapicWrapper(mpspec::mpc_ioapic);
#[derive(Copy, Clone)]
struct MpcTableWrapper(mpspec::mpc_table);
#[derive(Copy, Clone)]
struct MpcLintsrcWrapper(mpspec::mpc_lintsrc);
#[derive(Copy, Clone)]
struct MpfIntelWrapper(mpspec::mpf_intel);

// These `mpspec` wrapper types are only data, reading them from data is a safe initialization.
unsafe impl DataInit for MpcBusWrapper {}
unsafe impl DataInit for MpcCpuWrapper {}
unsafe impl DataInit for MpcIntsrcWrapper {}
unsafe impl DataInit for MpcIoapicWrapper {}
unsafe impl DataInit for MpcTableWrapper {}
unsafe impl DataInit for MpcLintsrcWrapper {}
unsafe impl DataInit for MpfIntelWrapper {}

// MPTABLE, describing VCPUS.
const MPTABLE_START: usize = 0x9fc00;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// There was too little guest memory to store the entire MP table.
    NotEnoughMemory,
    /// The MP table has too little address space to be stored.
    AddressOverflow,
    /// Failure while zeroing out the memory for the MP table.
    Clear,
    /// Number of CPUs exceeds the maximum supported CPUs
    TooManyCpus,
    /// Failure to write the MP floating pointer.
    WriteMpfIntel,
    /// Failure to write MP CPU entry.
    WriteMpcCpu,
    /// Failure to write MP ioapic entry.
    WriteMpcIoapic,
    /// Failure to write MP bus entry.
    WriteMpcBus,
    /// Failure to write MP interrupt source entry.
    WriteMpcIntsrc,
    /// Failure to write MP local interrupt source entry.
    WriteMpcLintsrc,
    /// Failure to write MP table header.
    WriteMpcTable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::NotEnoughMemory => write!(
                f,
                "There was too little guest memory to store the entire MP table."
            ),
            Error::AddressOverflow => {
                write!(f, "The MP table has too little address space to be stored.")
            }
            Error::Clear => write!(f, "Failure while zeroing out the memory for the MP table."),
            Error::TooManyCpus => write!(f, "Number of CPUs exceeds the maximum supported CPUs."),
            Error::WriteMpfIntel => write!(f, "Failure to write the MP floating pointer."),
            Error::WriteMpcCpu => write!(f, "Failure to write MP CPU entry."),
            Error::WriteMpcIoapic => write!(f, "Failure to write MP ioapic entry."),
            Error::WriteMpcBus => write!(f, "Failure to write MP bus entry."),
            Error::WriteMpcIntsrc => write!(f, "Failure to write MP interrupt source entry."),
            Error::WriteMpcLintsrc => {
                write!(f, "Failure to write MP local interrupt source entry.")
            }
            Error::WriteMpcTable => write!(f, "Failure to write MP table header."),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

// With APIC/xAPIC, there are only 255 APIC IDs available. And IOAPIC occupies
// one APIC ID, so only 254 CPUs at maximum may be supported. Actually it's
// a large number for FC usecases.
pub const MAX_SUPPORTED_CPUS: u32 = 254;

// Convenience macro for making arrays of diverse character types.
macro_rules! char_array {
    ($t:ty; $( $c:expr ),*) => ( [ $( $c as $t ),* ] )
}

// Most of these variables are sourced from the Intel MP Spec 1.4.
const SMP_MAGIC_IDENT: [c_char; 4] = char_array!(c_char; '_', 'M', 'P', '_');
const MPC_SIGNATURE: [c_char; 4] = char_array!(c_char; 'P', 'C', 'M', 'P');
const MPC_SPEC: i8 = 4;
const MPC_OEM: [c_char; 8] = char_array!(c_char; 'F', 'C', ' ', ' ', ' ', ' ', ' ', ' ');
const MPC_PRODUCT_ID: [c_char; 12] = ['0' as c_char; 12];
const BUS_TYPE_ISA: [u8; 6] = char_array!(u8; 'I', 'S', 'A', ' ', ' ', ' ');
const IO_APIC_DEFAULT_PHYS_BASE: u32 = 0xfec0_0000; // source: linux/arch/x86/include/asm/apicdef.h
const APIC_DEFAULT_PHYS_BASE: u32 = 0xfee0_0000; // source: linux/arch/x86/include/asm/apicdef.h
const APIC_VERSION: u8 = 0x14;
const CPU_STEPPING: u32 = 0x600;
const CPU_FEATURE_APIC: u32 = 0x200;
const CPU_FEATURE_FPU: u32 = 0x001;

fn compute_checksum<T: Copy>(v: &T) -> u8 {
    // Safe because we are only reading the bytes within the size of the `T` reference `v`.
    let v_slice = unsafe { slice::from_raw_parts(v as *const T as *const u8, mem::size_of::<T>()) };
    let mut checksum: u8 = 0;
    for i in v_slice.iter() {
        checksum = checksum.wrapping_add(*i);
    }
    checksum
}

fn mpf_intel_compute_checksum(v: &mpspec::mpf_intel) -> u8 {
    let checksum = compute_checksum(v).wrapping_sub(v.checksum);
    (!checksum).wrapping_add(1)
}

fn compute_mp_size(num_cpus: u8) -> usize {
    mem::size_of::<MpfIntelWrapper>()
        + mem::size_of::<MpcTableWrapper>()
        + mem::size_of::<MpcCpuWrapper>() * (num_cpus as usize)
        + mem::size_of::<MpcIoapicWrapper>()
        + mem::size_of::<MpcBusWrapper>()
        + mem::size_of::<MpcIntsrcWrapper>() * 16
        + mem::size_of::<MpcLintsrcWrapper>() * 2
}

/// Performs setup of the MP table for the given `num_cpus`.
pub fn setup_mptable(mem: &GuestMemory, num_cpus: u8) -> Result<()> {
    if u32::from(num_cpus) > MAX_SUPPORTED_CPUS {
        return Err(Error::TooManyCpus);
    }

    // Used to keep track of the next base pointer into the MP table.
    let mut base_mp = GuestAddress(MPTABLE_START);

    let mp_size = compute_mp_size(num_cpus);

    let mut checksum: u8 = 0;
    let ioapicid: u8 = num_cpus + 1;

    // The checked_add here ensures the all of the following base_mp.unchecked_add's will be without
    // overflow.
    if let Some(end_mp) = base_mp.checked_add(mp_size - 1) {
        if !mem.address_in_range(end_mp) {
            return Err(Error::NotEnoughMemory);
        }
    } else {
        return Err(Error::AddressOverflow);
    }

    mem.read_to_memory(base_mp, &mut io::repeat(0), mp_size)
        .map_err(|_| Error::Clear)?;

    {
        let mut mpf_intel = MpfIntelWrapper(mpspec::mpf_intel::default());
        let size = mem::size_of::<MpfIntelWrapper>();
        mpf_intel.0.signature = SMP_MAGIC_IDENT;
        mpf_intel.0.length = 1;
        mpf_intel.0.specification = 4;
        mpf_intel.0.physptr = (base_mp.offset() + size) as u32;
        mpf_intel.0.checksum = mpf_intel_compute_checksum(&mpf_intel.0);
        mem.write_obj_at_addr(mpf_intel, base_mp)
            .map_err(|_| Error::WriteMpfIntel)?;
        base_mp = base_mp.unchecked_add(size);
    }

    // We set the location of the mpc_table here but we can't fill it out until we have the length
    // of the entire table later.
    let table_base = base_mp;
    base_mp = base_mp.unchecked_add(mem::size_of::<MpcTableWrapper>());

    {
        let size = mem::size_of::<MpcCpuWrapper>();
        for cpu_id in 0..num_cpus {
            let mut mpc_cpu = MpcCpuWrapper(mpspec::mpc_cpu::default());
            mpc_cpu.0.type_ = mpspec::MP_PROCESSOR as u8;
            mpc_cpu.0.apicid = cpu_id;
            mpc_cpu.0.apicver = APIC_VERSION;
            mpc_cpu.0.cpuflag = mpspec::CPU_ENABLED as u8
                | if cpu_id == 0 {
                    mpspec::CPU_BOOTPROCESSOR as u8
                } else {
                    0
                };
            mpc_cpu.0.cpufeature = CPU_STEPPING;
            mpc_cpu.0.featureflag = CPU_FEATURE_APIC | CPU_FEATURE_FPU;
            mem.write_obj_at_addr(mpc_cpu, base_mp)
                .map_err(|_| Error::WriteMpcCpu)?;
            base_mp = base_mp.unchecked_add(size);
            checksum = checksum.wrapping_add(compute_checksum(&mpc_cpu.0));
        }
    }
    {
        let size = mem::size_of::<MpcBusWrapper>();
        let mut mpc_bus = MpcBusWrapper(mpspec::mpc_bus::default());
        mpc_bus.0.type_ = mpspec::MP_BUS as u8;
        mpc_bus.0.busid = 0;
        mpc_bus.0.bustype = BUS_TYPE_ISA;
        mem.write_obj_at_addr(mpc_bus, base_mp)
            .map_err(|_| Error::WriteMpcBus)?;
        base_mp = base_mp.unchecked_add(size);
        checksum = checksum.wrapping_add(compute_checksum(&mpc_bus.0));
    }
    {
        let size = mem::size_of::<MpcIoapicWrapper>();
        let mut mpc_ioapic = MpcIoapicWrapper(mpspec::mpc_ioapic::default());
        mpc_ioapic.0.type_ = mpspec::MP_IOAPIC as u8;
        mpc_ioapic.0.apicid = ioapicid;
        mpc_ioapic.0.apicver = APIC_VERSION;
        mpc_ioapic.0.flags = mpspec::MPC_APIC_USABLE as u8;
        mpc_ioapic.0.apicaddr = IO_APIC_DEFAULT_PHYS_BASE;
        mem.write_obj_at_addr(mpc_ioapic, base_mp)
            .map_err(|_| Error::WriteMpcIoapic)?;
        base_mp = base_mp.unchecked_add(size);
        checksum = checksum.wrapping_add(compute_checksum(&mpc_ioapic.0));
    }
    // Per kvm_setup_default_irq_routing() in kernel
    for i in 0..16 {
        let size = mem::size_of::<MpcIntsrcWrapper>();
        let mut mpc_intsrc = MpcIntsrcWrapper(mpspec::mpc_intsrc::default());
        mpc_intsrc.0.type_ = mpspec::MP_INTSRC as u8;
        mpc_intsrc.0.irqtype = mpspec::mp_irq_source_types_mp_INT as u8;
        mpc_intsrc.0.irqflag = mpspec::MP_IRQDIR_DEFAULT as u16;
        mpc_intsrc.0.srcbus = 0;
        mpc_intsrc.0.srcbusirq = i;
        mpc_intsrc.0.dstapic = ioapicid;
        mpc_intsrc.0.dstirq = i;
        mem.write_obj_at_addr(mpc_intsrc, base_mp)
            .map_err(|_| Error::WriteMpcIntsrc)?;
        base_mp = base_mp.unchecked_add(size);
        checksum = checksum.wrapping_add(compute_checksum(&mpc_intsrc.0));
    }
    {
        let size = mem::size_of::<MpcLintsrcWrapper>();
        let mut mpc_lintsrc = MpcLintsrcWrapper(mpspec::mpc_lintsrc::default());
        mpc_lintsrc.0.type_ = mpspec::MP_LINTSRC as u8;
        mpc_lintsrc.0.irqtype = mpspec::mp_irq_source_types_mp_ExtINT as u8;
        mpc_lintsrc.0.irqflag = mpspec::MP_IRQDIR_DEFAULT as u16;
        mpc_lintsrc.0.srcbusid = 0;
        mpc_lintsrc.0.srcbusirq = 0;
        mpc_lintsrc.0.destapic = 0;
        mpc_lintsrc.0.destapiclint = 0;
        mem.write_obj_at_addr(mpc_lintsrc, base_mp)
            .map_err(|_| Error::WriteMpcLintsrc)?;
        base_mp = base_mp.unchecked_add(size);
        checksum = checksum.wrapping_add(compute_checksum(&mpc_lintsrc.0));
    }
    {
        let size = mem::size_of::<MpcLintsrcWrapper>();
        let mut mpc_lintsrc = MpcLintsrcWrapper(mpspec::mpc_lintsrc::default());
        mpc_lintsrc.0.type_ = mpspec::MP_LINTSRC as u8;
        mpc_lintsrc.0.irqtype = mpspec::mp_irq_source_types_mp_NMI as u8;
        mpc_lintsrc.0.irqflag = mpspec::MP_IRQDIR_DEFAULT as u16;
        mpc_lintsrc.0.srcbusid = 0;
        mpc_lintsrc.0.srcbusirq = 0;
        mpc_lintsrc.0.destapic = 0xFF; /* to all local APICs */
        mpc_lintsrc.0.destapiclint = 1;
        mem.write_obj_at_addr(mpc_lintsrc, base_mp)
            .map_err(|_| Error::WriteMpcLintsrc)?;
        base_mp = base_mp.unchecked_add(size);
        checksum = checksum.wrapping_add(compute_checksum(&mpc_lintsrc.0));
    }

    // At this point we know the size of the mp_table.
    let table_end = base_mp;

    {
        let mut mpc_table = MpcTableWrapper(mpspec::mpc_table::default());
        mpc_table.0.signature = MPC_SIGNATURE;
        mpc_table.0.length = table_end.offset_from(table_base) as u16;
        mpc_table.0.spec = MPC_SPEC;
        mpc_table.0.oem = MPC_OEM;
        mpc_table.0.productid = MPC_PRODUCT_ID;
        mpc_table.0.lapic = APIC_DEFAULT_PHYS_BASE;
        checksum = checksum.wrapping_add(compute_checksum(&mpc_table.0));
        mpc_table.0.checksum = (!checksum).wrapping_add(1) as i8;
        mem.write_obj_at_addr(mpc_table, table_base)
            .map_err(|_| Error::WriteMpcTable)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_entry_size(type_: u8) -> usize {
        match u32::from(type_) {
            mpspec::MP_PROCESSOR => mem::size_of::<MpcCpuWrapper>(),
            mpspec::MP_BUS => mem::size_of::<MpcBusWrapper>(),
            mpspec::MP_IOAPIC => mem::size_of::<MpcIoapicWrapper>(),
            mpspec::MP_INTSRC => mem::size_of::<MpcIntsrcWrapper>(),
            mpspec::MP_LINTSRC => mem::size_of::<MpcLintsrcWrapper>(),
            _ => panic!("unrecognized mpc table entry type: {}", type_),
        }
    }

    #[test]
    fn bounds_check() {
        let num_cpus = 4;
        let mem = GuestMemory::new(&[(GuestAddress(MPTABLE_START), compute_mp_size(num_cpus))])
            .unwrap_or_else(|err| panic!("{}", err));

        setup_mptable(&mem, num_cpus).unwrap();
    }

    #[test]
    fn bounds_check_fails() {
        let num_cpus = 4;
        let mem = GuestMemory::new(&[(GuestAddress(MPTABLE_START), compute_mp_size(num_cpus) - 1)])
            .unwrap_or_else(|err| panic!("{}", err));

        assert!(setup_mptable(&mem, num_cpus).is_err());
    }

    #[test]
    fn mpf_intel_checksum() {
        let num_cpus = 1;
        let mem = GuestMemory::new(&[(GuestAddress(MPTABLE_START), compute_mp_size(num_cpus))])
            .unwrap_or_else(|err| panic!("{}", err));

        setup_mptable(&mem, num_cpus).unwrap_or_else(|err| panic!("{}", err));

        let mpf_intel: MpfIntelWrapper = mem
            .read_obj_from_addr(GuestAddress(MPTABLE_START))
            .unwrap_or_else(|err| panic!("{}", err));

        assert_eq!(
            mpf_intel_compute_checksum(&mpf_intel.0),
            mpf_intel.0.checksum
        );
    }

    #[test]
    fn mpc_table_checksum() {
        let num_cpus = 4;
        let mem = GuestMemory::new(&[(GuestAddress(MPTABLE_START), compute_mp_size(num_cpus))])
            .unwrap_or_else(|err| panic!("{}", err));

        setup_mptable(&mem, num_cpus).unwrap_or_else(|err| panic!("{}", err));

        let mpf_intel: MpfIntelWrapper = mem
            .read_obj_from_addr(GuestAddress(MPTABLE_START))
            .unwrap_or_else(|err| panic!("{}", err));
        let mpc_offset = GuestAddress(mpf_intel.0.physptr as usize);
        let mpc_table: MpcTableWrapper = mem
            .read_obj_from_addr(mpc_offset)
            .unwrap_or_else(|err| panic!("{}", err));

        struct Sum(u8);
        impl io::Write for Sum {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                for v in buf.iter() {
                    self.0 = self.0.wrapping_add(*v);
                }
                Ok(buf.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut sum = Sum(0);
        mem.write_from_memory(mpc_offset, &mut sum, mpc_table.0.length as usize)
            .unwrap_or_else(|err| panic!("{}", err));
        assert_eq!(sum.0, 0);
    }

    #[test]
    fn cpu_entry_count() {
        let mem = GuestMemory::new(&[(
            GuestAddress(MPTABLE_START),
            compute_mp_size(MAX_SUPPORTED_CPUS as u8),
        )])
        .unwrap_or_else(|err| panic!("{}", err));

        for i in 0..MAX_SUPPORTED_CPUS as u8 {
            setup_mptable(&mem, i).unwrap_or_else(|err| panic!("{}", err));

            let mpf_intel: MpfIntelWrapper = mem
                .read_obj_from_addr(GuestAddress(MPTABLE_START))
                .unwrap_or_else(|err| panic!("{}", err));
            let mpc_offset = GuestAddress(mpf_intel.0.physptr as usize);
            let mpc_table: MpcTableWrapper = mem
                .read_obj_from_addr(mpc_offset)
                .unwrap_or_else(|err| panic!("{}", err));
            let mpc_end = mpc_offset.checked_add(mpc_table.0.length as usize).unwrap();

            let mut entry_offset = mpc_offset
                .checked_add(mem::size_of::<MpcTableWrapper>())
                .unwrap();
            let mut cpu_count = 0;
            while entry_offset < mpc_end {
                let entry_type: u8 = mem
                    .read_obj_from_addr(entry_offset)
                    .unwrap_or_else(|err| panic!("{}", err));
                entry_offset = entry_offset
                    .checked_add(table_entry_size(entry_type))
                    .unwrap();
                assert!(entry_offset <= mpc_end);
                if u32::from(entry_type) == mpspec::MP_PROCESSOR {
                    cpu_count += 1;
                }
            }
            assert_eq!(cpu_count, i);
        }
    }

    #[test]
    fn cpu_entry_count_max() {
        let cpus = MAX_SUPPORTED_CPUS + 1;
        let mem = GuestMemory::new(&[(GuestAddress(MPTABLE_START), compute_mp_size(cpus as u8))])
            .unwrap_or_else(|err| panic!("{}", err));

        let result = setup_mptable(&mem, cpus as u8).unwrap_err();
        assert_eq!(result, Error::TooManyCpus);
    }
}
