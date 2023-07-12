// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Handles routing to devices in an address space.

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::collections::btree_map::BTreeMap;
use std::result;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    #[error("New device overlaps with an old device.")]
    Overlap,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
struct BusRange(u64, u64);

impl Eq for BusRange {}

impl PartialEq for BusRange {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, other))]
    fn eq(&self, other: &BusRange) -> bool {
        self.0 == other.0
    }
}

impl Ord for BusRange {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, other))]
    fn cmp(&self, other: &BusRange) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for BusRange {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, other))]
    fn partial_cmp(&self, other: &BusRange) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// A device container for routing reads and writes over some address space.
///
/// This doesn't have any restrictions on what kind of device or address space this applies to. The
/// only restriction is that no two devices can overlap in this address space.
#[derive(Debug, Clone, Default)]
pub struct Bus {
    devices: BTreeMap<BusRange, Arc<Mutex<BusDevice>>>,
}

use event_manager::{EventOps, Events, MutEventSubscriber};

#[cfg(target_arch = "aarch64")]
use super::legacy::RTCDevice;
use super::legacy::{I8042Device, SerialDevice};
use super::pseudo::BootTimer;
use super::virtio::MmioTransport;

#[derive(Debug)]
pub enum BusDevice {
    I8042Device(I8042Device),
    #[cfg(target_arch = "aarch64")]
    RTCDevice(RTCDevice),
    BootTimer(BootTimer),
    MmioTransport(MmioTransport),
    Serial(SerialDevice<std::io::Stdin>),
    #[cfg(test)]
    Dummy(DummyDevice),
    #[cfg(test)]
    Constant(ConstantDevice),
}

#[cfg(test)]
#[derive(Debug)]
pub struct DummyDevice;

#[cfg(test)]
impl DummyDevice {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, _offset, _data))]
    pub fn bus_write(&mut self, _offset: u64, _data: &[u8]) {}
    #[tracing::instrument(level = "debug", ret(skip), skip(self, _offset, _data))]
    pub fn bus_read(&mut self, _offset: u64, _data: &[u8]) {}
}

#[cfg(test)]
#[derive(Debug)]
pub struct ConstantDevice;

#[cfg(test)]
impl ConstantDevice {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, offset, data))]
    pub fn bus_read(&mut self, offset: u64, data: &mut [u8]) {
        for (i, v) in data.iter_mut().enumerate() {
            *v = (offset as u8) + (i as u8);
        }
    }

    #[tracing::instrument(level = "debug", ret(skip), skip(self, offset, data))]
    fn bus_write(&mut self, offset: u64, data: &[u8]) {
        for (i, v) in data.iter().enumerate() {
            assert_eq!(*v, (offset as u8) + (i as u8))
        }
    }
}

impl BusDevice {
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn i8042_device_ref(&self) -> Option<&I8042Device> {
        match self {
            Self::I8042Device(x) => Some(x),
            _ => None,
        }
    }
    #[cfg(target_arch = "aarch64")]
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn rtc_device_ref(&self) -> Option<&RTCDevice> {
        match self {
            Self::RTCDevice(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn boot_timer_ref(&self) -> Option<&BootTimer> {
        match self {
            Self::BootTimer(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn mmio_transport_ref(&self) -> Option<&MmioTransport> {
        match self {
            Self::MmioTransport(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn serial_ref(&self) -> Option<&SerialDevice<std::io::Stdin>> {
        match self {
            Self::Serial(x) => Some(x),
            _ => None,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn i8042_device_mut(&mut self) -> Option<&mut I8042Device> {
        match self {
            Self::I8042Device(x) => Some(x),
            _ => None,
        }
    }
    #[cfg(target_arch = "aarch64")]
    #[tracing::instrument(level = "debug", ret(skip), skip(self))]
    pub fn rtc_device_mut(&mut self) -> Option<&mut RTCDevice> {
        match self {
            Self::RTCDevice(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn boot_timer_mut(&mut self) -> Option<&mut BootTimer> {
        match self {
            Self::BootTimer(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn mmio_transport_mut(&mut self) -> Option<&mut MmioTransport> {
        match self {
            Self::MmioTransport(x) => Some(x),
            _ => None,
        }
    }
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn serial_mut(&mut self) -> Option<&mut SerialDevice<std::io::Stdin>> {
        match self {
            Self::Serial(x) => Some(x),
            _ => None,
        }
    }

    #[tracing::instrument(level = "debug", ret(skip), skip(self, offset, data))]
    pub fn read(&mut self, offset: u64, data: &mut [u8]) {
        match self {
            Self::I8042Device(x) => x.bus_read(offset, data),
            #[cfg(target_arch = "aarch64")]
            Self::RTCDevice(x) => x.bus_read(offset, data),
            Self::BootTimer(x) => x.bus_read(offset, data),
            Self::MmioTransport(x) => x.bus_read(offset, data),
            Self::Serial(x) => x.bus_read(offset, data),
            #[cfg(test)]
            Self::Dummy(x) => x.bus_read(offset, data),
            #[cfg(test)]
            Self::Constant(x) => x.bus_read(offset, data),
        }
    }

    #[tracing::instrument(level = "debug", ret(skip), skip(self, offset, data))]
    pub fn write(&mut self, offset: u64, data: &[u8]) {
        match self {
            Self::I8042Device(x) => x.bus_write(offset, data),
            #[cfg(target_arch = "aarch64")]
            Self::RTCDevice(x) => x.bus_write(offset, data),
            Self::BootTimer(x) => x.bus_write(offset, data),
            Self::MmioTransport(x) => x.bus_write(offset, data),
            Self::Serial(x) => x.bus_write(offset, data),
            #[cfg(test)]
            Self::Dummy(x) => x.bus_write(offset, data),
            #[cfg(test)]
            Self::Constant(x) => x.bus_write(offset, data),
        }
    }
}

impl MutEventSubscriber for BusDevice {
    #[tracing::instrument(level = "debug", ret(skip), skip(self, event, ops))]
    fn process(&mut self, event: Events, ops: &mut EventOps) {
        match self {
            Self::Serial(serial) => serial.process(event, ops),
            _ => panic!(),
        }
    }
    #[tracing::instrument(level = "debug", ret(skip), skip(self, ops))]
    fn init(&mut self, ops: &mut EventOps) {
        match self {
            Self::Serial(serial) => serial.init(ops),
            _ => panic!(),
        }
    }
}

impl Bus {
    /// Constructs an a bus with an empty address space.
    #[tracing::instrument(level = "debug", ret(skip), skip())]
    pub fn new() -> Bus {
        Bus {
            devices: BTreeMap::new(),
        }
    }

    #[tracing::instrument(level = "debug", ret(skip), skip(self, addr))]
    fn first_before(&self, addr: u64) -> Option<(BusRange, &Mutex<BusDevice>)> {
        // for when we switch to rustc 1.17: self.devices.range(..addr).iter().rev().next()
        for (range, dev) in self.devices.iter().rev() {
            if range.0 <= addr {
                return Some((*range, dev));
            }
        }
        None
    }

    #[tracing::instrument(level = "debug", ret(skip), skip(self, addr))]
    pub fn get_device(&self, addr: u64) -> Option<(u64, &Mutex<BusDevice>)> {
        if let Some((BusRange(start, len), dev)) = self.first_before(addr) {
            let offset = addr - start;
            if offset < len {
                return Some((offset, dev));
            }
        }
        None
    }

    /// Puts the given device at the given address space.
    #[tracing::instrument(level = "debug", ret(skip), skip(self, device, base, len))]
    pub fn insert(&mut self, device: Arc<Mutex<BusDevice>>, base: u64, len: u64) -> Result<()> {
        if len == 0 {
            return Err(Error::Overlap);
        }

        // Reject all cases where the new device's base is within an old device's range.
        if self.get_device(base).is_some() {
            return Err(Error::Overlap);
        }

        // The above check will miss an overlap in which the new device's base address is before the
        // range of another device. To catch that case, we search for a device with a range before
        // the new device's range's end. If there is no existing device in that range that starts
        // after the new device, then there will be no overlap.
        if let Some((BusRange(start, _), _)) = self.first_before(base + len - 1) {
            // Such a device only conflicts with the new device if it also starts after the new
            // device because of our initial `get_device` check above.
            if start >= base {
                return Err(Error::Overlap);
            }
        }

        if self.devices.insert(BusRange(base, len), device).is_some() {
            return Err(Error::Overlap);
        }

        Ok(())
    }

    /// Reads data from the device that owns the range containing `addr` and puts it into `data`.
    ///
    /// Returns true on success, otherwise `data` is untouched.
    #[tracing::instrument(level = "debug", ret(skip), skip(self, addr, data))]
    pub fn read(&self, addr: u64, data: &mut [u8]) -> bool {
        if let Some((offset, dev)) = self.get_device(addr) {
            // OK to unwrap as lock() failing is a serious error condition and should panic.
            dev.lock()
                .expect("Failed to acquire device lock")
                .read(offset, data);
            true
        } else {
            false
        }
    }

    /// Writes `data` to the device that owns the range containing `addr`.
    ///
    /// Returns true on success, otherwise `data` is untouched.
    #[tracing::instrument(level = "debug", ret(skip), skip(self, addr, data))]
    pub fn write(&self, addr: u64, data: &[u8]) -> bool {
        if let Some((offset, dev)) = self.get_device(addr) {
            // OK to unwrap as lock() failing is a serious error condition and should panic.
            dev.lock()
                .expect("Failed to acquire device lock")
                .write(offset, data);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_insert() {
        let mut bus = Bus::new();
        let dummy = Arc::new(Mutex::new(BusDevice::Dummy(DummyDevice)));
        // Insert len should not be 0.
        assert!(bus.insert(dummy.clone(), 0x10, 0).is_err());
        assert!(bus.insert(dummy.clone(), 0x10, 0x10).is_ok());

        let result = bus.insert(dummy.clone(), 0x0f, 0x10);
        // This overlaps the address space of the existing bus device at 0x10.
        assert!(result.is_err());
        assert_eq!(format!("{:?}", result), "Err(Overlap)");

        // This overlaps the address space of the existing bus device at 0x10.
        assert!(bus.insert(dummy.clone(), 0x10, 0x10).is_err());
        // This overlaps the address space of the existing bus device at 0x10.
        assert!(bus.insert(dummy.clone(), 0x10, 0x15).is_err());
        // This overlaps the address space of the existing bus device at 0x10.
        assert!(bus.insert(dummy.clone(), 0x12, 0x15).is_err());
        // This overlaps the address space of the existing bus device at 0x10.
        assert!(bus.insert(dummy.clone(), 0x12, 0x01).is_err());
        // This overlaps the address space of the existing bus device at 0x10.
        assert!(bus.insert(dummy.clone(), 0x0, 0x20).is_err());
        assert!(bus.insert(dummy.clone(), 0x20, 0x05).is_ok());
        assert!(bus.insert(dummy.clone(), 0x25, 0x05).is_ok());
        assert!(bus.insert(dummy, 0x0, 0x10).is_ok());
    }

    #[test]
    fn bus_read_write() {
        let mut bus = Bus::new();
        let dummy = Arc::new(Mutex::new(BusDevice::Dummy(DummyDevice)));
        assert!(bus.insert(dummy, 0x10, 0x10).is_ok());
        assert!(bus.read(0x10, &mut [0, 0, 0, 0]));
        assert!(bus.write(0x10, &[0, 0, 0, 0]));
        assert!(bus.read(0x11, &mut [0, 0, 0, 0]));
        assert!(bus.write(0x11, &[0, 0, 0, 0]));
        assert!(bus.read(0x16, &mut [0, 0, 0, 0]));
        assert!(bus.write(0x16, &[0, 0, 0, 0]));
        assert!(!bus.read(0x20, &mut [0, 0, 0, 0]));
        assert!(!bus.write(0x20, &[0, 0, 0, 0]));
        assert!(!bus.read(0x06, &mut [0, 0, 0, 0]));
        assert!(!bus.write(0x06, &[0, 0, 0, 0]));
    }

    #[test]
    fn bus_read_write_values() {
        let mut bus = Bus::new();
        let dummy = Arc::new(Mutex::new(BusDevice::Constant(ConstantDevice)));
        assert!(bus.insert(dummy, 0x10, 0x10).is_ok());

        let mut values = [0, 1, 2, 3];
        assert!(bus.read(0x10, &mut values));
        assert_eq!(values, [0, 1, 2, 3]);
        assert!(bus.write(0x10, &values));
        assert!(bus.read(0x15, &mut values));
        assert_eq!(values, [5, 6, 7, 8]);
        assert!(bus.write(0x15, &values));
    }

    #[test]
    fn busrange_cmp_and_clone() {
        assert_eq!(BusRange(0x10, 2), BusRange(0x10, 3));
        assert_eq!(BusRange(0x10, 2), BusRange(0x10, 2));

        assert!(BusRange(0x10, 2) < BusRange(0x12, 1));
        assert!(BusRange(0x10, 2) < BusRange(0x12, 3));

        let mut bus = Bus::new();
        let mut data = [1, 2, 3, 4];
        assert!(bus
            .insert(
                Arc::new(Mutex::new(BusDevice::Dummy(DummyDevice))),
                0x10,
                0x10
            )
            .is_ok());
        assert!(bus.write(0x10, &data));
        let bus_clone = bus.clone();
        assert!(bus.read(0x10, &mut data));
        assert_eq!(data, [1, 2, 3, 4]);
        assert!(bus_clone.read(0x10, &mut data));
        assert_eq!(data, [1, 2, 3, 4]);
    }

    #[test]
    fn test_display_error() {
        assert_eq!(
            format!("{}", Error::Overlap),
            "New device overlaps with an old device."
        );
    }
}
