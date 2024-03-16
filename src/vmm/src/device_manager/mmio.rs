// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use kvm_ioctls::{IoEventAddress, VmFd};
use log::info;
use serde::{Deserialize, Serialize};
use vm_allocator::{AddressAllocator, AllocPolicy, IdAllocator};

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::DeviceInfoForFDT;
use crate::arch::DeviceType;
use crate::arch::DeviceType::Virtio;
use crate::devices::virtio::balloon::Balloon;
use crate::devices::virtio::block::device::Block;
use crate::devices::virtio::device::VirtioDevice;
use crate::devices::virtio::mmio::MmioTransport;
use crate::devices::virtio::net::Net;
use crate::devices::virtio::rng::Entropy;
use crate::devices::virtio::vsock::TYPE_VSOCK;
use crate::devices::virtio::{TYPE_BALLOON, TYPE_BLOCK, TYPE_NET, TYPE_RNG};
use crate::devices::BusDevice;

/// Errors for MMIO device manager.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum MmioError {
    /// Failed to allocate requested resource: {0}
    Allocator(vm_allocator::Error),
    /// Failed to insert device on the bus: {0}
    BusInsert(crate::devices::BusError),
    /// Failed to find the device on the bus.
    DeviceNotFound,
    /// Invalid device type found on the MMIO bus.
    InvalidDeviceType,
    /// {0}
    InternalDeviceError(String),
    /// Invalid MMIO IRQ configuration.
    InvalidIrqConfig,
    /// Failed to register IO event: {0}
    RegisterIoEvent(kvm_ioctls::Error),
    /// Failed to register irqfd: {0}
    RegisterIrqFd(kvm_ioctls::Error),
}

/// This represents the size of the mmio device specified to the kernel as a cmdline option
/// It has to be larger than 0x100 (the offset where the configuration space starts from
/// the beginning of the memory mapped device registers) + the size of the configuration space
/// Currently hardcoded to 4K.
pub const MMIO_LEN: u64 = 0x1000;

/// Stores the address range and irq allocated to this device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MMIODeviceInfo {
    /// Mmio address at which the device is registered.
    pub addr: u64,
    /// Mmio addr range length.
    pub len: u64,
    /// Used Irq line(s) for the device.
    pub irqs: Vec<u32>,
}

impl MMIODeviceInfo {
    pub fn register_kvm_device(
        &self,
        vm: &VmFd,
        mmio_device: &MmioTransport,
    ) -> Result<(), MmioError> {
        let locked_device = mmio_device.locked_device();
        for (i, queue_evt) in locked_device.queue_events().iter().enumerate() {
            let io_addr = IoEventAddress::Mmio(
                self.addr + u64::from(crate::devices::virtio::NOTIFY_REG_OFFSET),
            );
            vm.register_ioevent(queue_evt, &io_addr, u32::try_from(i).unwrap())
                .map_err(MmioError::RegisterIoEvent)?;
        }
        vm.register_irqfd(locked_device.interrupt_evt(), self.irqs[0])
            .map_err(MmioError::RegisterIrqFd)?;
        Ok(())
    }
}

/// Manages the complexities of registering a MMIO device.
#[derive(Debug)]
pub struct MMIODeviceManager {
    pub(crate) bus: crate::devices::Bus,
    pub(crate) irq_allocator: IdAllocator,
    pub(crate) address_allocator: AddressAllocator,
    pub(crate) id_to_dev_info: HashMap<(DeviceType, String), MMIODeviceInfo>,
}

impl MMIODeviceManager {
    /// Create a new DeviceManager handling mmio devices (virtio net, block).
    pub fn new(
        mmio_base: u64,
        mmio_size: u64,
        (irq_start, irq_end): (u32, u32),
    ) -> Result<MMIODeviceManager, MmioError> {
        Ok(MMIODeviceManager {
            irq_allocator: IdAllocator::new(irq_start, irq_end).map_err(MmioError::Allocator)?,
            address_allocator: AddressAllocator::new(mmio_base, mmio_size)
                .map_err(MmioError::Allocator)?,
            bus: crate::devices::Bus::new(),
            id_to_dev_info: HashMap::new(),
        })
    }

    /// Allocates resources for a new device to be added.
    fn allocate_mmio_resources(&mut self, irq_count: u32) -> Result<MMIODeviceInfo, MmioError> {
        let irqs = (0..irq_count)
            .map(|_| self.irq_allocator.allocate_id())
            .collect::<vm_allocator::Result<_>>()
            .map_err(MmioError::Allocator)?;
        let device_info = MMIODeviceInfo {
            addr: self
                .address_allocator
                .allocate(MMIO_LEN, MMIO_LEN, AllocPolicy::FirstMatch)
                .map_err(MmioError::Allocator)?
                .start(),
            len: MMIO_LEN,
            irqs,
        };
        Ok(device_info)
    }

    /// Add new virtio-over-MMIO device.
    pub fn add_device(
        &mut self,
        vm: &VmFd,
        device_id: String,
        mmio_device: MmioTransport,
    ) -> Result<MMIODeviceInfo, MmioError> {
        let device_info = self.allocate_mmio_resources(1)?;
        self.register_mmio_virtio(vm, device_id, mmio_device, &device_info)?;
        Ok(device_info)
    }

    /// Add new MMIO device to the MMIO bus.
    pub fn add_bus_device(
        &mut self,
        identifier: (DeviceType, String),
        device: Arc<Mutex<BusDevice>>,
    ) -> Result<(), MmioError> {
        let device_info = self.allocate_mmio_resources(1)?;
        self.add_bus_device_with_info(identifier, device, device_info)
    }

    /// Add new MMIO device to the MMIO bus with specified
    /// device info.
    pub fn add_bus_device_with_info(
        &mut self,
        identifier: (DeviceType, String),
        device: Arc<Mutex<BusDevice>>,
        device_info: MMIODeviceInfo,
    ) -> Result<(), MmioError> {
        self.bus
            .insert(device, device_info.addr, device_info.len)
            .map_err(MmioError::BusInsert)?;
        self.id_to_dev_info.insert(identifier, device_info);
        Ok(())
    }

    /// Register a virtio-over-MMIO device to be used via MMIO transport at a specific slot.
    pub fn register_mmio_virtio(
        &mut self,
        vm: &VmFd,
        device_id: String,
        mmio_device: MmioTransport,
        device_info: &MMIODeviceInfo,
    ) -> Result<(), MmioError> {
        // Our virtio devices are currently hardcoded to use a single IRQ.
        // Validate that requirement.
        if device_info.irqs.len() != 1 {
            return Err(MmioError::InvalidIrqConfig);
        }
        let identifier = {
            let locked_device = mmio_device.locked_device();
            (DeviceType::Virtio(locked_device.device_type()), device_id)
        };

        device_info.register_kvm_device(vm, &mmio_device)?;

        self.add_bus_device_with_info(
            identifier,
            Arc::new(Mutex::new(BusDevice::MmioTransport(mmio_device))),
            device_info.clone(),
        )
    }

    #[cfg(target_arch = "aarch64")]
    /// Register an early console at the specified MMIO configuration if given as parameter,
    /// otherwise allocate a new MMIO resources for it.
    pub fn register_mmio_serial(
        &mut self,
        vm: &VmFd,
        serial: Arc<Mutex<BusDevice>>,
        device_info_opt: Option<MMIODeviceInfo>,
    ) -> Result<MMIODeviceInfo, MmioError> {
        // Create a new MMIODeviceInfo object on boot path or unwrap the
        // existing object on restore path.
        let device_info = if let Some(device_info) = device_info_opt {
            device_info
        } else {
            self.allocate_mmio_resources(1)?
        };

        vm.register_irqfd(
            serial
                .lock()
                .expect("Poisoned lock")
                .serial_ref()
                .unwrap()
                .serial
                .interrupt_evt(),
            device_info.irqs[0],
        )
        .map_err(MmioError::RegisterIrqFd)?;

        let identifier = (DeviceType::Serial, DeviceType::Serial.to_string());
        // Register the newly created Serial object.
        self.add_bus_device_with_info(identifier, serial, device_info.clone())?;
        Ok(device_info)
    }

    /// Gets the information of the devices registered up to some point in time.
    pub fn get_device_info(&self) -> &HashMap<(DeviceType, String), MMIODeviceInfo> {
        &self.id_to_dev_info
    }

    /// Gets the specified device.
    pub fn get_device(
        &self,
        device_type: DeviceType,
        device_id: &str,
    ) -> Option<&Mutex<BusDevice>> {
        if let Some(device_info) = self
            .id_to_dev_info
            .get(&(device_type, device_id.to_string()))
        {
            if let Some((_, device)) = self.bus.get_device(device_info.addr) {
                return Some(device);
            }
        }
        None
    }

    /// Run fn for each registered device.
    pub fn for_each_device<F, E: Debug>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&DeviceType, &String, &MMIODeviceInfo, &Mutex<BusDevice>) -> Result<(), E>,
    {
        for ((device_type, device_id), device_info) in self.get_device_info().iter() {
            let bus_device = self
                .get_device(*device_type, device_id)
                // Safe to unwrap() because we know the device exists.
                .unwrap();
            f(device_type, device_id, device_info, bus_device)?;
        }
        Ok(())
    }

    /// Run fn for each registered virtio device.
    pub fn for_each_virtio_device<F, E: Debug>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(u32, &String, &MMIODeviceInfo, Arc<Mutex<dyn VirtioDevice>>) -> Result<(), E>,
    {
        self.for_each_device(|device_type, device_id, device_info, bus_device| {
            if let Virtio(virtio_type) = device_type {
                let virtio_device = bus_device
                    .lock()
                    .expect("Poisoned lock")
                    .mmio_transport_ref()
                    .expect("Unexpected device type")
                    .device();
                f(*virtio_type, device_id, device_info, virtio_device)?;
            }
            Ok(())
        })?;

        Ok(())
    }

    /// Run fn `f()` for the virtio device matching `virtio_type` and `id`.
    pub fn with_virtio_device_with_id<T, F>(
        &self,
        virtio_type: u32,
        id: &str,
        f: F,
    ) -> Result<(), MmioError>
    where
        T: VirtioDevice + 'static + Debug,
        F: FnOnce(&mut T) -> Result<(), String>,
    {
        if let Some(busdev) = self.get_device(DeviceType::Virtio(virtio_type), id) {
            let virtio_device = busdev
                .lock()
                .expect("Poisoned lock")
                .mmio_transport_ref()
                .expect("Unexpected device type")
                .device();
            let mut dev = virtio_device.lock().expect("Poisoned lock");
            f(dev
                .as_mut_any()
                .downcast_mut::<T>()
                .ok_or(MmioError::InvalidDeviceType)?)
            .map_err(MmioError::InternalDeviceError)?;
        } else {
            return Err(MmioError::DeviceNotFound);
        }
        Ok(())
    }

    /// Artificially kick devices as if they had external events.
    pub fn kick_devices(&self) {
        info!("Artificially kick devices.");
        // We only kick virtio devices for now.
        let _: Result<(), MmioError> =
            self.for_each_virtio_device(|virtio_type, id, _info, dev| {
                let mut virtio = dev.lock().expect("Poisoned lock");
                match virtio_type {
                    TYPE_BALLOON => {
                        let balloon = virtio.as_mut_any().downcast_mut::<Balloon>().unwrap();
                        // If device is activated, kick the balloon queue(s) to make up for any
                        // pending or in-flight epoll events we may have not captured in snapshot.
                        // Stats queue doesn't need kicking as it is notified via a `timer_fd`.
                        if balloon.is_activated() {
                            info!("kick balloon {}.", id);
                            balloon.process_virtio_queues();
                        }
                    }
                    TYPE_BLOCK => {
                        // We only care about kicking virtio block.
                        // If we need to kick vhost-user-block we can do nothing.
                        if let Some(block) = virtio.as_mut_any().downcast_mut::<Block>() {
                            // If device is activated, kick the block queue(s) to make up for any
                            // pending or in-flight epoll events we may have not captured in
                            // snapshot. No need to kick Ratelimiters
                            // because they are restored 'unblocked' so
                            // any inflight `timer_fd` events can be safely discarded.
                            if block.is_activated() {
                                info!("kick block {}.", id);
                                block.process_virtio_queues();
                            }
                        }
                    }
                    TYPE_NET => {
                        let net = virtio.as_mut_any().downcast_mut::<Net>().unwrap();
                        // If device is activated, kick the net queue(s) to make up for any
                        // pending or in-flight epoll events we may have not captured in snapshot.
                        // No need to kick Ratelimiters because they are restored 'unblocked' so
                        // any inflight `timer_fd` events can be safely discarded.
                        if net.is_activated() {
                            info!("kick net {}.", id);
                            net.process_virtio_queues();
                        }
                    }
                    TYPE_VSOCK => {
                        // Vsock has complicated protocol that isn't resilient to any packet loss,
                        // so for Vsock we don't support connection persistence through snapshot.
                        // Any in-flight packets or events are simply lost.
                        // Vsock is restored 'empty'.
                    }
                    TYPE_RNG => {
                        let entropy = virtio.as_mut_any().downcast_mut::<Entropy>().unwrap();
                        if entropy.is_activated() {
                            info!("kick entropy {id}.");
                            entropy.process_virtio_queues();
                        }
                    }
                    _ => (),
                }
                Ok(())
            });
    }
}

#[cfg(target_arch = "aarch64")]
impl DeviceInfoForFDT for MMIODeviceInfo {
    fn addr(&self) -> u64 {
        self.addr
    }
    fn irq(&self) -> u32 {
        self.irqs[0]
    }
    fn length(&self) -> u64 {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;

    use utils::eventfd::EventFd;

    use super::*;
    use crate::devices::virtio::device::VirtioDevice;
    use crate::devices::virtio::queue::Queue;
    use crate::devices::virtio::ActivateError;
    use crate::utilities::test_utils::multi_region_mem;
    use crate::vstate::memory::{GuestAddress, GuestMemoryMmap};
    use crate::{builder, Vm};

    const QUEUE_SIZES: &[u16] = &[64];

    impl MMIODeviceManager {
        fn register_virtio_test_device(
            &mut self,
            vm: &VmFd,
            guest_mem: GuestMemoryMmap,
            device: Arc<Mutex<dyn VirtioDevice>>,
            cmdline: &mut kernel_cmdline::Cmdline,
            dev_id: &str,
        ) -> Result<u64, MmioError> {
            let mmio_device = MmioTransport::new(guest_mem, device, false);
            let device_info =
                self.register_mmio_virtio_for_boot(vm, dev_id.to_string(), mmio_device, cmdline)?;
            Ok(device_info.addr)
        }

        #[cfg(target_arch = "x86_64")]
        /// Gets the number of interrupts used by the devices registered.
        pub fn used_irqs_count(&self) -> usize {
            let mut irq_number = 0;
            self.get_device_info()
                .iter()
                .for_each(|(_, device_info)| irq_number += device_info.irqs.len());
            irq_number
        }
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct DummyDevice {
        dummy: u32,
        queues: Vec<Queue>,
        queue_evts: [EventFd; 1],
        interrupt_evt: EventFd,
    }

    impl DummyDevice {
        pub fn new() -> Self {
            DummyDevice {
                dummy: 0,
                queues: QUEUE_SIZES.iter().map(|&s| Queue::new(s)).collect(),
                queue_evts: [EventFd::new(libc::EFD_NONBLOCK).expect("cannot create eventFD")],
                interrupt_evt: EventFd::new(libc::EFD_NONBLOCK).expect("cannot create eventFD"),
            }
        }
    }

    impl VirtioDevice for DummyDevice {
        fn avail_features(&self) -> u64 {
            0
        }

        fn acked_features(&self) -> u64 {
            0
        }

        fn set_acked_features(&mut self, _: u64) {}

        fn device_type(&self) -> u32 {
            0
        }

        fn queues(&self) -> &[Queue] {
            &self.queues
        }

        fn queues_mut(&mut self) -> &mut [Queue] {
            &mut self.queues
        }

        fn queue_events(&self) -> &[EventFd] {
            &self.queue_evts
        }

        fn interrupt_evt(&self) -> &EventFd {
            &self.interrupt_evt
        }

        fn interrupt_status(&self) -> Arc<AtomicU32> {
            Arc::new(AtomicU32::new(0))
        }

        fn ack_features_by_page(&mut self, page: u32, value: u32) {
            let _ = page;
            let _ = value;
        }

        fn read_config(&self, offset: u64, data: &mut [u8]) {
            let _ = offset;
            let _ = data;
        }

        fn write_config(&mut self, offset: u64, data: &[u8]) {
            let _ = offset;
            let _ = data;
        }

        fn activate(&mut self, _: GuestMemoryMmap) -> Result<(), ActivateError> {
            Ok(())
        }

        fn is_activated(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_register_virtio_device() {
        let start_addr1 = GuestAddress(0x0);
        let start_addr2 = GuestAddress(0x1000);
        let guest_mem = multi_region_mem(&[(start_addr1, 0x1000), (start_addr2, 0x1000)]);
        let mut vm = Vm::new(vec![]).unwrap();
        vm.memory_init(&guest_mem, false).unwrap();
        let mut device_manager = MMIODeviceManager::new(
            0xd000_0000,
            crate::arch::MMIO_MEM_SIZE,
            (crate::arch::IRQ_BASE, crate::arch::IRQ_MAX),
        )
        .unwrap();

        let mut cmdline = kernel_cmdline::Cmdline::new(4096).unwrap();
        let dummy = Arc::new(Mutex::new(DummyDevice::new()));
        #[cfg(target_arch = "x86_64")]
        builder::setup_interrupt_controller(&mut vm).unwrap();
        #[cfg(target_arch = "aarch64")]
        builder::setup_interrupt_controller(&mut vm, 1).unwrap();

        device_manager
            .register_virtio_test_device(vm.fd(), guest_mem, dummy, &mut cmdline, "dummy")
            .unwrap();
    }

    #[test]
    fn test_register_too_many_devices() {
        let start_addr1 = GuestAddress(0x0);
        let start_addr2 = GuestAddress(0x1000);
        let guest_mem = multi_region_mem(&[(start_addr1, 0x1000), (start_addr2, 0x1000)]);
        let mut vm = Vm::new(vec![]).unwrap();
        vm.memory_init(&guest_mem, false).unwrap();
        let mut device_manager = MMIODeviceManager::new(
            0xd000_0000,
            crate::arch::MMIO_MEM_SIZE,
            (crate::arch::IRQ_BASE, crate::arch::IRQ_MAX),
        )
        .unwrap();

        let mut cmdline = kernel_cmdline::Cmdline::new(4096).unwrap();
        #[cfg(target_arch = "x86_64")]
        builder::setup_interrupt_controller(&mut vm).unwrap();
        #[cfg(target_arch = "aarch64")]
        builder::setup_interrupt_controller(&mut vm, 1).unwrap();

        for _i in crate::arch::IRQ_BASE..=crate::arch::IRQ_MAX {
            device_manager
                .register_virtio_test_device(
                    vm.fd(),
                    guest_mem.clone(),
                    Arc::new(Mutex::new(DummyDevice::new())),
                    &mut cmdline,
                    "dummy1",
                )
                .unwrap();
        }
        assert_eq!(
            format!(
                "{}",
                device_manager
                    .register_virtio_test_device(
                        vm.fd(),
                        guest_mem,
                        Arc::new(Mutex::new(DummyDevice::new())),
                        &mut cmdline,
                        "dummy2"
                    )
                    .unwrap_err()
            ),
            "Failed to allocate requested resource: The requested resource is not available."
                .to_string()
        );
    }

    #[test]
    fn test_dummy_device() {
        let dummy = DummyDevice::new();
        assert_eq!(dummy.device_type(), 0);
        assert_eq!(dummy.queues().len(), QUEUE_SIZES.len());
    }

    #[test]
    fn test_device_info() {
        let start_addr1 = GuestAddress(0x0);
        let start_addr2 = GuestAddress(0x1000);
        let guest_mem = multi_region_mem(&[(start_addr1, 0x1000), (start_addr2, 0x1000)]);
        let mut vm = Vm::new(vec![]).unwrap();
        vm.memory_init(&guest_mem, false).unwrap();

        let mem_clone = guest_mem.clone();

        #[cfg(target_arch = "x86_64")]
        builder::setup_interrupt_controller(&mut vm).unwrap();
        #[cfg(target_arch = "aarch64")]
        builder::setup_interrupt_controller(&mut vm, 1).unwrap();

        let mut device_manager = MMIODeviceManager::new(
            0xd000_0000,
            crate::arch::MMIO_MEM_SIZE,
            (crate::arch::IRQ_BASE, crate::arch::IRQ_MAX),
        )
        .unwrap();
        let mut cmdline = kernel_cmdline::Cmdline::new(4096).unwrap();
        let dummy = Arc::new(Mutex::new(DummyDevice::new()));

        let type_id = dummy.lock().unwrap().device_type();
        let id = String::from("foo");
        let addr = device_manager
            .register_virtio_test_device(vm.fd(), guest_mem, dummy, &mut cmdline, &id)
            .unwrap();
        assert!(device_manager
            .get_device(DeviceType::Virtio(type_id), &id)
            .is_some());
        assert_eq!(
            addr,
            device_manager.id_to_dev_info[&(DeviceType::Virtio(type_id), id.clone())].addr
        );
        assert_eq!(
            crate::arch::IRQ_BASE,
            device_manager.id_to_dev_info[&(DeviceType::Virtio(type_id), id)].irqs[0]
        );

        let id = "bar";
        assert!(device_manager
            .get_device(DeviceType::Virtio(type_id), id)
            .is_none());

        let dummy2 = Arc::new(Mutex::new(DummyDevice::new()));
        let id2 = String::from("foo2");
        device_manager
            .register_virtio_test_device(vm.fd(), mem_clone, dummy2, &mut cmdline, &id2)
            .unwrap();

        let mut count = 0;
        let _: Result<(), MmioError> = device_manager.for_each_device(|devtype, devid, _, _| {
            assert_eq!(*devtype, DeviceType::Virtio(type_id));
            match devid.as_str() {
                "foo" => count += 1,
                "foo2" => count += 2,
                _ => unreachable!(),
            };
            Ok(())
        });
        assert_eq!(count, 3);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(device_manager.used_irqs_count(), 2);
    }

    #[test]
    fn test_slot_irq_allocation() {
        let mut device_manager = MMIODeviceManager::new(
            0xd000_0000,
            crate::arch::MMIO_MEM_SIZE,
            (crate::arch::IRQ_BASE, crate::arch::IRQ_MAX),
        )
        .unwrap();
        let device_info = device_manager.allocate_mmio_resources(0).unwrap();
        assert_eq!(device_info.irqs.len(), 0);
        let device_info = device_manager.allocate_mmio_resources(1).unwrap();
        assert_eq!(device_info.irqs[0], crate::arch::IRQ_BASE);
        assert_eq!(
            format!(
                "{}",
                device_manager
                    .allocate_mmio_resources(crate::arch::IRQ_MAX - crate::arch::IRQ_BASE + 1)
                    .unwrap_err()
            ),
            "Failed to allocate requested resource: The requested resource is not available."
                .to_string()
        );

        for i in crate::arch::IRQ_BASE..crate::arch::IRQ_MAX {
            device_manager.irq_allocator.free_id(i).unwrap();
        }

        let device_info = device_manager
            .allocate_mmio_resources(crate::arch::IRQ_MAX - crate::arch::IRQ_BASE - 1)
            .unwrap();
        assert_eq!(device_info.irqs[16], crate::arch::IRQ_BASE + 16);
        assert_eq!(
            format!("{}", device_manager.allocate_mmio_resources(2).unwrap_err()),
            "Failed to allocate requested resource: The requested resource is not available."
                .to_string()
        );
        device_manager.allocate_mmio_resources(0).unwrap();
    }
}
