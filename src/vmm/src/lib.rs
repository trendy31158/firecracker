// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Virtual Machine Monitor that leverages the Linux Kernel-based Virtual Machine (KVM),
//! and other virtualization features to run a single lightweight micro-virtual
//! machine (microVM).
#![deny(missing_docs)]
extern crate epoll;
extern crate kvm_bindings;
extern crate kvm_ioctls;
extern crate libc;
extern crate polly;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate arch;
#[cfg(target_arch = "x86_64")]
extern crate cpuid;
extern crate devices;
extern crate kernel;
#[macro_use]
extern crate logger;
extern crate dumbo;
extern crate memory_model;
extern crate rate_limiter;
extern crate seccomp;
extern crate utils;

/// Handles setup and initialization a `Vmm` object.
pub mod builder;
/// Handles runtime configuration of a `Vmm` object.
pub mod controller;
/// Syscalls allowed through the seccomp filter.
pub mod default_syscalls;
pub(crate) mod device_manager;
/// Resource store for configured microVM resources.
pub mod resources;
/// microVM RPC API adapters.
pub mod rpc_interface;
/// Signal handling utilities.
pub mod signal_handler;
/// Wrappers over structures used to configure the VMM.
pub mod vmm_config;
mod vstate;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use arch::DeviceType;
#[cfg(target_arch = "x86_64")]
use device_manager::legacy::PortIODeviceManager;
#[cfg(target_arch = "aarch64")]
use device_manager::mmio::MMIODeviceInfo;
use device_manager::mmio::MMIODeviceManager;
use devices::virtio::EpollConfigConstructor;
use devices::{BusDevice, DeviceEventT, EpollHandler};
use kernel::cmdline::Cmdline as KernelCmdline;
use logger::error::LoggerError;
#[cfg(target_arch = "x86_64")]
use logger::LogOption;
use logger::{Metric, LOGGER, METRICS};
use memory_model::GuestMemory;
use polly::event_manager::{self, EventHandler};
use polly::pollable::{Pollable, PollableOp, PollableOpBuilder};
use utils::eventfd::EventFd;
use utils::time::TimestampUs;
use vstate::{Vcpu, Vm};

/// Success exit code.
pub const FC_EXIT_CODE_OK: u8 = 0;
/// Generic error exit code.
pub const FC_EXIT_CODE_GENERIC_ERROR: u8 = 1;
/// Generic exit code for an error considered not possible to occur if the program logic is sound.
pub const FC_EXIT_CODE_UNEXPECTED_ERROR: u8 = 2;
/// Firecracker was shut down after intercepting a restricted system call.
pub const FC_EXIT_CODE_BAD_SYSCALL: u8 = 148;
/// Firecracker was shut down after intercepting `SIGBUS`.
pub const FC_EXIT_CODE_SIGBUS: u8 = 149;
/// Firecracker was shut down after intercepting `SIGSEGV`.
pub const FC_EXIT_CODE_SIGSEGV: u8 = 150;
/// Bad configuration for microvm's resources, when using a single json.
pub const FC_EXIT_CODE_BAD_CONFIGURATION: u8 = 151;

/// Dispatch categories for epoll events.
pub enum EpollDispatch {
    /// Event has to be dispatch to an EpollHandler.
    DeviceHandler(usize, DeviceEventT),
}

struct MaybeHandler {
    handler: Option<Box<dyn EpollHandler>>,
    receiver: Receiver<Box<dyn EpollHandler>>,
}

impl MaybeHandler {
    fn new(receiver: Receiver<Box<dyn EpollHandler>>) -> Self {
        MaybeHandler {
            handler: None,
            receiver,
        }
    }
}

/// Handles epoll related business.
// A glaring shortcoming of the current design is the liberal passing around of raw_fds,
// and duping of file descriptors. This issue will be solved when we also implement device removal.
pub struct EpollContext {
    epoll_raw_fd: RawFd,
    // FIXME: find a different design as this does not scale. This Vec can only grow.
    dispatch_table: Vec<Option<EpollDispatch>>,
    device_handlers: Vec<MaybeHandler>,
    device_id_to_handler_id: HashMap<(u32, String), usize>,

    // This part of the class relates to incoming epoll events. The incoming events are held in
    // `events[event_index..num_events)`, followed by the events not yet read from `epoll_raw_fd`.
    events: Vec<epoll::Event>,
    num_events: usize,
    event_index: usize,
}

impl EpollContext {
    /// Creates a new `EpollContext` object.
    pub fn new() -> Result<Self> {
        const EPOLL_EVENTS_LEN: usize = 100;

        let epoll_raw_fd = epoll::create(true).map_err(Error::EpollFd)?;

        // Initial capacity needs to be large enough to hold:
        // * 1 exit event
        // * 2 queue events for virtio block
        // * 4 for virtio net
        // The total is 8 elements; allowing spare capacity to avoid reallocations.
        let mut dispatch_table = Vec::with_capacity(20);
        dispatch_table.push(None);
        Ok(EpollContext {
            epoll_raw_fd,
            dispatch_table,
            device_handlers: Vec::with_capacity(6),
            device_id_to_handler_id: HashMap::new(),
            events: vec![epoll::Event::new(epoll::Events::empty(), 0); EPOLL_EVENTS_LEN],
            num_events: 0,
            event_index: 0,
        })
    }

    /// Given a file descriptor `fd`, and an EpollDispatch token `token`,
    /// associate `token` with an `EPOLLIN` event for `fd`, through the
    /// `dispatch_table`.
    pub fn add_epollin_event<T: AsRawFd + ?Sized>(
        &mut self,
        fd: &T,
        token: EpollDispatch,
    ) -> Result<()> {
        // The index in the dispatch where the new token will be added.
        let dispatch_index = self.dispatch_table.len() as u64;

        // Add a new epoll event on `fd`, associated with index
        // `dispatch_index`.
        epoll::ctl(
            self.epoll_raw_fd,
            epoll::ControlOptions::EPOLL_CTL_ADD,
            fd.as_raw_fd(),
            epoll::Event::new(epoll::Events::EPOLLIN, dispatch_index),
        )
        .map_err(Error::EpollFd)?;

        // Add the associated token at index `dispatch_index`
        self.dispatch_table.push(Some(token));

        Ok(())
    }

    /// Allocates `count` dispatch tokens, simultaneously registering them in
    /// `dispatch_table`. The tokens will be associated with a device.
    /// This device's handler will be added to the end of `device_handlers`.
    /// This returns the index of the first token, and a channel on which to
    /// send an epoll handler for the relevant device.
    pub fn allocate_tokens_for_device(
        &mut self,
        count: usize,
    ) -> (u64, Sender<Box<dyn EpollHandler>>) {
        let dispatch_base = self.dispatch_table.len() as u64;
        let device_idx = self.device_handlers.len();
        let (sender, receiver) = channel();

        self.dispatch_table.extend((0..count).map(|index| {
            Some(EpollDispatch::DeviceHandler(
                device_idx,
                index as DeviceEventT,
            ))
        }));
        self.device_handlers.push(MaybeHandler::new(receiver));

        (dispatch_base, sender)
    }

    /// Allocate tokens for a virtio device, as with `allocate_tokens_for_device`,
    /// but also call T::new to create a device handler for the device. This handler
    /// will then be associated to a given `device_id` through the `device_id_to_handler_id`
    /// table. Finally, return the handler.
    pub fn allocate_tokens_for_virtio_device<T: EpollConfigConstructor>(
        &mut self,
        type_id: u32,
        device_id: &str,
        count: usize,
    ) -> T {
        let (dispatch_base, sender) = self.allocate_tokens_for_device(count);

        self.device_id_to_handler_id.insert(
            (type_id, device_id.to_string()),
            self.device_handlers.len() - 1,
        );

        T::new(dispatch_base, self.epoll_raw_fd, sender)
    }

    /// Obtains the `EpollHandler` trait object associated with the provided handler id.
    pub fn get_device_handler_by_handler_id(&mut self, id: usize) -> Result<&mut dyn EpollHandler> {
        let maybe = &mut self.device_handlers[id];
        match maybe.handler {
            Some(ref mut v) => Ok(v.as_mut()),
            None => {
                // This should only be called in response to an epoll trigger.
                // Moreover, this branch of the match should only be active on the first call
                // (the first epoll event for this device), therefore the channel is guaranteed
                // to contain a message for the first epoll event since both epoll event
                // registration and channel send() happen in the device activate() function.
                let received = maybe
                    .receiver
                    .try_recv()
                    .map_err(|_| Error::DeviceEventHandlerNotFound)?;
                Ok(maybe.handler.get_or_insert(received).as_mut())
            }
        }
    }

    /// Obtains a mut reference to an object implementing `EpollHandler` for the given device
    /// type and identifier.
    pub fn get_device_handler_by_device_id<T: EpollHandler + 'static>(
        &mut self,
        type_id: u32,
        device_id: &str,
    ) -> Result<&mut T> {
        let handler_id = *self
            .device_id_to_handler_id
            .get(&(type_id, device_id.to_string()))
            .ok_or(Error::DeviceEventHandlerNotFound)?;
        let device_handler = self.get_device_handler_by_handler_id(handler_id)?;
        device_handler
            .as_mut_any()
            .downcast_mut::<T>()
            .ok_or(Error::DeviceEventHandlerInvalidDowncast)
    }

    /// Gets the next event from `epoll_raw_fd`.
    pub fn get_event(&mut self) -> Result<epoll::Event> {
        // Check if no events are left in `events`:
        while self.num_events == self.event_index {
            // If so, get more events.
            // Note that if there is an error, we propagate it.
            self.num_events =
                epoll::wait(self.epoll_raw_fd, -1, &mut self.events[..]).map_err(Error::Poll)?;
            // And reset the event_index.
            self.event_index = 0;
        }

        // Now, move our position in the stream.
        self.event_index += 1;

        // And return the appropriate event.
        Ok(self.events[self.event_index - 1])
    }

    /// Wait for and dispatch events.
    pub fn run_event_loop(&mut self) {
        let event = self.get_event().unwrap();
        let evset = match epoll::Events::from_bits(event.events) {
            Some(evset) => evset,
            None => {
                let evbits = event.events;
                warn!("epoll: ignoring unknown event set: 0x{:x}", evbits);
                return;
            }
        };

        match self.dispatch_table[event.data as usize] {
            Some(EpollDispatch::DeviceHandler(device_idx, device_token)) => {
                METRICS.vmm.device_events.inc();
                match self.get_device_handler_by_handler_id(device_idx) {
                    Ok(handler) => match handler.handle_event(device_token, evset) {
                        Err(devices::Error::PayloadExpected) => {
                            panic!("Received update disk image event with empty payload.")
                        }
                        Err(devices::Error::UnknownEvent { device, event }) => {
                            panic!("Unknown event: {:?} {:?}", device, event)
                        }
                        _ => (),
                    },
                    Err(e) => warn!("invalid handler for device {}: {:?}", device_idx, e),
                }
            }
            None => {
                panic!("what do you mean nothing?!");
                // Do nothing.
            }
        };
        // Currently, we never get to return with Ok(EventLoopExitReason::Break) because
        // we just invoke stop() whenever that would happen.
    }
}

impl AsRawFd for EpollContext {
    fn as_raw_fd(&self) -> RawFd {
        self.epoll_raw_fd
    }
}

impl EventHandler for EpollContext {
    /// Handle a read event (EPOLLIN).
    fn handle_read(&mut self, source: Pollable) -> Vec<PollableOp> {
        if source == self.epoll_raw_fd {
            self.run_event_loop();
        } else {
            error!("Spurious EventManager event for handler: EpollContext");
        }
        vec![]
    }

    fn init(&self) -> Vec<PollableOp> {
        vec![PollableOpBuilder::new(self.as_raw_fd())
            .readable()
            .register()]
    }
}

impl Drop for EpollContext {
    fn drop(&mut self) {
        let rc = unsafe { libc::close(self.epoll_raw_fd) };
        if rc != 0 {
            warn!("Cannot close epoll.");
        }
    }
}

/// Errors associated with the VMM internal logic. These errors cannot be generated by direct user
/// input, but can result from bad configuration of the host (for example if Firecracker doesn't
/// have permissions to open the KVM fd).
#[derive(Debug)]
pub enum Error {
    /// This error is thrown by the minimal boot loader implementation.
    ConfigureSystem(arch::Error),
    /// Legacy devices work with Event file descriptors and the creation can fail because
    /// of resource exhaustion.
    #[cfg(target_arch = "x86_64")]
    CreateLegacyDevice(device_manager::legacy::Error),
    /// An operation on the epoll instance failed due to resource exhaustion or bad configuration.
    EpollFd(io::Error),
    /// Cannot read from an Event file descriptor.
    EventFd(io::Error),
    /// Polly error wrapper.
    EventManager(event_manager::Error),
    /// An event arrived for a device, but the dispatcher can't find the event (epoll) handler.
    DeviceEventHandlerNotFound,
    /// An epoll handler can't be downcasted to the desired type.
    DeviceEventHandlerInvalidDowncast,
    /// I8042 Error.
    I8042Error(devices::legacy::I8042DeviceError),
    /// Cannot access kernel file.
    KernelFile(io::Error),
    /// Cannot open /dev/kvm. Either the host does not have KVM or Firecracker does not have
    /// permission to open the file descriptor.
    KvmContext(vstate::Error),
    #[cfg(target_arch = "x86_64")]
    /// Cannot add devices to the Legacy I/O Bus.
    LegacyIOBus(device_manager::legacy::Error),
    /// Cannot load command line.
    LoadCommandline(kernel::cmdline::Error),
    /// Internal logger error.
    Logger(LoggerError),
    /// Epoll wait failed.
    Poll(io::Error),
    /// Cannot add a device to the MMIO Bus.
    RegisterMMIODevice(device_manager::mmio::Error),
    /// Cannot build seccomp filters.
    SeccompFilters(seccomp::Error),
    /// Write to the serial console failed.
    Serial(io::Error),
    /// Cannot create Timer file descriptor.
    TimerFd(io::Error),
    /// Vcpu error.
    Vcpu(vstate::Error),
    /// Cannot spawn a new Vcpu thread.
    VcpuSpawn(std::io::Error),
    /// Vm error.
    Vm(vstate::Error),
    /// Error thrown by observer object on Vmm initialization.
    VmmObserverInit(utils::errno::Error),
    /// Error thrown by observer object on Vmm teardown.
    VmmObserverTeardown(utils::errno::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::Error::*;

        match self {
            ConfigureSystem(e) => write!(f, "System configuration error: {:?}", e),
            #[cfg(target_arch = "x86_64")]
            CreateLegacyDevice(e) => write!(f, "Error creating legacy device: {:?}", e),
            EpollFd(e) => write!(f, "Epoll fd error: {}", e),
            EventFd(e) => write!(f, "Event fd error: {}", e),
            EventManager(e) => write!(f, "Event manager error: {:?}", e),
            DeviceEventHandlerNotFound => write!(
                f,
                "Device event handler not found. This might point to a guest device driver issue."
            ),
            DeviceEventHandlerInvalidDowncast => write!(
                f,
                "Device event handler couldn't be downcasted to expected type."
            ),
            I8042Error(e) => write!(f, "I8042 error: {}", e),
            KernelFile(e) => write!(f, "Cannot access kernel file: {}", e),
            KvmContext(e) => write!(f, "Failed to validate KVM support: {:?}", e),
            #[cfg(target_arch = "x86_64")]
            LegacyIOBus(e) => write!(f, "Cannot add devices to the legacy I/O Bus. {}", e),
            LoadCommandline(e) => write!(f, "Cannot load command line: {}", e),
            Logger(e) => write!(f, "Logger error: {}", e),
            Poll(e) => write!(f, "Epoll wait failed: {}", e),
            RegisterMMIODevice(e) => write!(f, "Cannot add a device to the MMIO Bus. {}", e),
            SeccompFilters(e) => write!(f, "Cannot build seccomp filters: {}", e),
            Serial(e) => write!(f, "Error writing to the serial console: {:?}", e),
            TimerFd(e) => write!(f, "Error creating timer fd: {}", e),
            Vcpu(e) => write!(f, "Vcpu error: {}", e),
            VcpuSpawn(e) => write!(f, "Cannot spawn Vcpu thread: {}", e),
            Vm(e) => write!(f, "Vm error: {}", e),
            VmmObserverInit(e) => write!(
                f,
                "Error thrown by observer object on Vmm initialization: {}",
                e
            ),
            VmmObserverTeardown(e) => {
                write!(f, "Error thrown by observer object on Vmm teardown: {}", e)
            }
        }
    }
}

/// Trait for objects that need custom initialization and teardown during the Vmm lifetime.
pub trait VmmEventsObserver {
    /// This function will be called during microVm boot.
    fn on_vmm_boot(&mut self) -> std::result::Result<(), utils::errno::Error> {
        Ok(())
    }
    /// This function will be called on microVm teardown.
    fn on_vmm_stop(&mut self) -> std::result::Result<(), utils::errno::Error> {
        Ok(())
    }
}

/// Shorthand result type for internal VMM commands.
pub type Result<T> = std::result::Result<T, Error>;

/// Contains the state and associated methods required for the Firecracker VMM.
pub struct Vmm {
    events_observer: Option<Box<dyn VmmEventsObserver>>,

    // Guest VM core resources.
    guest_memory: GuestMemory,

    kernel_cmdline: KernelCmdline,

    vcpus_handles: Vec<thread::JoinHandle<()>>,
    exit_evt: EventFd,
    vm: Vm,

    // Guest VM devices.
    mmio_device_manager: MMIODeviceManager,
    #[cfg(target_arch = "x86_64")]
    pio_device_manager: PortIODeviceManager,

    // The level of seccomp filtering used. Seccomp filters are loaded before executing guest code.
    seccomp_level: u32,
}

impl Vmm {
    /// Gets the the specified bus device.
    pub fn get_bus_device(
        &self,
        device_type: DeviceType,
        device_id: &str,
    ) -> Option<&Mutex<dyn BusDevice>> {
        self.mmio_device_manager.get_device(device_type, device_id)
    }

    #[cfg(target_arch = "x86_64")]
    fn log_dirty_pages(&mut self) {
        // If we're logging dirty pages, post the metrics on how many dirty pages there are.
        if LOGGER.flags() | LogOption::LogDirtyPages as usize > 0 {
            METRICS.memory.dirty_pages.add(self.get_dirty_page_count());
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn get_mmio_device_info(&self) -> Option<&HashMap<(DeviceType, String), MMIODeviceInfo>> {
        Some(self.mmio_device_manager.get_device_info())
    }

    fn start_vcpus(&mut self, mut vcpus: Vec<Vcpu>) -> Result<()> {
        let vcpu_count = vcpus.len();

        if let Some(observer) = self.events_observer.as_mut() {
            observer.on_vmm_boot().map_err(Error::VmmObserverInit)?;
        }

        Vcpu::register_kick_signal_handler();

        self.vcpus_handles.reserve(vcpu_count as usize);

        let vcpus_thread_barrier = Arc::new(Barrier::new((vcpu_count + 1) as usize));

        // We're going in reverse so we can `.pop()` on the vec and still maintain order.
        for cpu_id in (0..vcpu_count).rev() {
            let vcpu_thread_barrier = vcpus_thread_barrier.clone();

            let vcpu_exit_evt = self.exit_evt.try_clone().map_err(Error::EventFd)?;

            // `unwrap` is safe since we are asserting that the `vcpu_count` is equal to the number
            // of items of `vcpus` vector.
            let mut vcpu = vcpus.pop().unwrap();

            vcpu.set_mmio_bus(self.mmio_device_manager.bus.clone());

            let seccomp_level = self.seccomp_level;
            self.vcpus_handles.push(
                thread::Builder::new()
                    .name(format!("fc_vcpu{}", cpu_id))
                    .spawn(move || {
                        vcpu.run(vcpu_thread_barrier, seccomp_level, vcpu_exit_evt);
                    })
                    .map_err(Error::VcpuSpawn)?,
            );
        }

        // Load seccomp filters for the VMM thread.
        // Execution panics if filters cannot be loaded, use --seccomp-level=0 if skipping filters
        // altogether is the desired behaviour.
        default_syscalls::set_seccomp_level(self.seccomp_level).map_err(Error::SeccompFilters)?;

        vcpus_thread_barrier.wait();

        Ok(())
    }

    #[allow(unused_variables)]
    fn configure_system(&self, vcpus: &[Vcpu]) -> Result<()> {
        #[cfg(target_arch = "x86_64")]
        arch::x86_64::configure_system(
            &self.guest_memory,
            memory_model::GuestAddress(arch::x86_64::layout::CMDLINE_START),
            self.kernel_cmdline.len() + 1,
            vcpus.len() as u8,
        )
        .map_err(Error::ConfigureSystem)?;

        #[cfg(target_arch = "aarch64")]
        {
            let vcpu_mpidr = vcpus.into_iter().map(|cpu| cpu.get_mpidr()).collect();
            arch::aarch64::configure_system(
                &self.guest_memory,
                &self
                    .kernel_cmdline
                    .as_cstring()
                    .map_err(Error::LoadCommandline)?,
                vcpu_mpidr,
                self.get_mmio_device_info(),
                self.vm.get_irqchip(),
            )
            .map_err(Error::ConfigureSystem)?;
        }
        Ok(())
    }

    /// Returns a reference to the inner `GuestMemory` object if present, or `None` otherwise.
    pub fn guest_memory(&self) -> &GuestMemory {
        &self.guest_memory
    }

    /// Injects CTRL+ALT+DEL keystroke combo in the i8042 device.
    #[cfg(target_arch = "x86_64")]
    pub fn send_ctrl_alt_del(&mut self) -> Result<()> {
        self.pio_device_manager
            .i8042
            .lock()
            .expect("i8042 lock was poisoned")
            .trigger_ctrl_alt_del()
            .map_err(Error::I8042Error)
    }

    /// Waits for all vCPUs to exit and terminates the Firecracker process.
    pub fn stop(&mut self, exit_code: i32) {
        info!("Vmm is stopping.");

        if let Some(observer) = self.events_observer.as_mut() {
            if let Err(e) = observer.on_vmm_stop() {
                warn!("{}", Error::VmmObserverTeardown(e));
            }
        }

        // Log the metrics before exiting.
        if let Err(e) = LOGGER.log_metrics() {
            error!("Failed to log metrics while stopping: {}", e);
        }

        // Exit from Firecracker using the provided exit code. Safe because we're terminating
        // the process anyway.
        unsafe {
            libc::_exit(exit_code);
        }
    }

    // Count the number of pages dirtied since the last call to this function.
    // Because this is used for metrics, it swallows most errors and simply doesn't count dirty
    // pages if the KVM operation fails.
    #[cfg(target_arch = "x86_64")]
    fn get_dirty_page_count(&mut self) -> usize {
        let dirty_pages_in_region =
            |(slot, memory_region): (usize, &memory_model::MemoryRegion)| {
                self.vm
                    .fd()
                    .get_dirty_log(slot as u32, memory_region.size())
                    .map(|v| v.iter().map(|page| page.count_ones() as usize).sum())
                    .unwrap_or(0 as usize)
            };

        self.guest_memory()
            .map_and_fold(0, dirty_pages_in_region, std::ops::Add::add)
    }

    fn log_boot_time(t0_ts: &TimestampUs) {
        let now_tm_us = TimestampUs::default();

        let boot_time_us = now_tm_us.time_us - t0_ts.time_us;
        let boot_time_cpu_us = now_tm_us.cputime_us - t0_ts.cputime_us;
        info!(
            "Guest-boot-time = {:>6} us {} ms, {:>6} CPU us {} CPU ms",
            boot_time_us,
            boot_time_us / 1000,
            boot_time_cpu_us,
            boot_time_cpu_us / 1000
        );
    }

    /// Returns a reference to the inner KVM Vm object.
    pub fn kvm_vm(&self) -> &Vm {
        &self.vm
    }
}

impl EventHandler for Vmm {
    /// Handle a read event (EPOLLIN).
    fn handle_read(&mut self, source: Pollable) -> Vec<PollableOp> {
        if source == self.exit_evt.as_raw_fd() {
            let _ = self.exit_evt.read();
            self.stop(i32::from(FC_EXIT_CODE_OK));
        } else {
            error!("Spurious EventManager event for handler: Vmm");
        }
        vec![]
    }

    fn init(&self) -> Vec<PollableOp> {
        vec![PollableOpBuilder::new(self.exit_evt.as_raw_fd())
            .readable()
            .register()]
    }
}

#[cfg(test)]
mod tests {}

/*
    macro_rules! assert_match {
        ($x:expr, $y:pat) => {{
            if let $y = $x {
                ()
            } else {
                panic!()
            }
        }};
    }

    use super::*;

    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::sync::atomic::AtomicUsize;

    use arch::DeviceType;
    use devices::virtio::{ActivateResult, MmioTransport, Queue};
    use dumbo::MacAddr;
    use vmm_config::drive::DriveError;
    use vmm_config::machine_config::CpuFeaturesTemplate;
    use vmm_config::{RateLimiterConfig, TokenBucketConfig};

    fn good_kernel_file() -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let parent = path.parent().unwrap();

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        return [parent.to_str().unwrap(), "kernel/src/loader/test_elf.bin"]
            .iter()
            .collect();
        #[cfg(target_arch = "aarch64")]
        return [parent.to_str().unwrap(), "kernel/src/loader/test_pe.bin"]
            .iter()
            .collect();
    }

    impl Vmm {
        fn get_kernel_cmdline_str(&self) -> &str {
            if let Some(ref k) = self.kernel_config {
                k.cmdline.as_str()
            } else {
                ""
            }
        }

        fn remove_device_info(&mut self, type_id: u32, id: &str) {
            self.mmio_device_manager
                .as_mut()
                .unwrap()
                .remove_device_info(type_id, id);
        }

        fn default_kernel_config(&mut self, cust_kernel_path: Option<PathBuf>) {
            let kernel_temp_file =
                NamedTempFile::new().expect("Failed to create temporary kernel file.");
            let kernel_path = match cust_kernel_path {
                Some(kernel_path) => kernel_path,
                None => kernel_temp_file.path().to_path_buf(),
            };
            let kernel_file = File::open(kernel_path).expect("Cannot open kernel file");
            let mut cmdline = kernel_cmdline::Cmdline::new(arch::CMDLINE_MAX_SIZE);
            assert!(cmdline.insert_str(DEFAULT_KERNEL_CMDLINE).is_ok());
            let kernel_cfg = KernelConfig {
                cmdline,
                kernel_file,
            };
            self.set_kernel_config(kernel_cfg);
        }

        fn set_instance_state(&mut self, instance_state: InstanceState) {
            self.shared_info.write().unwrap().state = instance_state;
        }

        fn update_block_device_path(&mut self, block_device_id: &str, new_path: PathBuf) {
            for config in self.device_configs.block.config_list.iter_mut() {
                if config.drive_id == block_device_id {
                    config.path_on_host = new_path;
                    break;
                }
            }
        }

        fn change_id(&mut self, prev_id: &str, new_id: &str) {
            for config in self.device_configs.block.config_list.iter_mut() {
                if config.drive_id == prev_id {
                    config.drive_id = new_id.to_string();
                    break;
                }
            }
        }
    }

    struct DummyEpollHandler {
        evt: Option<DeviceEventT>,
    }

    impl EpollHandler for DummyEpollHandler {
        fn handle_event(
            &mut self,
            device_event: DeviceEventT,
            _evset: epoll::Events,
        ) -> std::result::Result<(), devices::Error> {
            self.evt = Some(device_event);
            Ok(())
        }
    }

    #[allow(dead_code)]
    #[derive(Clone)]
    struct DummyDevice {
        dummy: u32,
    }

    impl devices::virtio::VirtioDevice for DummyDevice {
        fn device_type(&self) -> u32 {
            0
        }

        fn queue_max_sizes(&self) -> &[u16] {
            &[10]
        }

        fn ack_features_by_page(&mut self, _: u32, _: u32) {}

        fn avail_features(&self) -> u64 {
            0
        }

        fn acked_features(&self) -> u64 {
            0
        }

        fn set_acked_features(&mut self, _: u64) {}

        fn read_config(&self, offset: u64, data: &mut [u8]) {
            let _ = offset;
            let _ = data;
        }

        fn write_config(&mut self, offset: u64, data: &[u8]) {
            let _ = offset;
            let _ = data;
        }

        #[allow(unused_variables)]
        #[allow(unused_mut)]
        fn activate(
            &mut self,
            mem: GuestMemory,
            interrupt_evt: EventFd,
            status: Arc<AtomicUsize>,
            queues: Vec<devices::virtio::Queue>,
            mut queue_evts: Vec<EventFd>,
        ) -> ActivateResult {
            Ok(())
        }
    }

    fn create_vmm_object(state: InstanceState) -> Vmm {
        let shared_info = Arc::new(RwLock::new(InstanceInfo {
            state,
            id: "TEST_ID".to_string(),
            vmm_version: "1.0".to_string(),
        }));

        Vmm::new(
            shared_info,
            &EventFd::new(libc::EFD_NONBLOCK).expect("Cannot create eventFD"),
            seccomp::SECCOMP_LEVEL_NONE,
        )
        .expect("Cannot Create VMM")
    }

    #[test]
    fn test_device_handler() {
        let mut ep = EpollContext::new().unwrap();
        let (base, sender) = ep.allocate_tokens_for_device(1);
        assert_eq!(ep.device_handlers.len(), 1);
        assert_eq!(base, 1);

        let handler = DummyEpollHandler { evt: None };
        assert!(sender.send(Box::new(handler)).is_ok());
        assert!(ep.get_device_handler_by_handler_id(0).is_ok());
    }



    #[test]
    fn test_insert_net_device() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        // test create network interface
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif"),
            host_dev_name: String::from("hostname1"),
            guest_mac: None,
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };
        assert!(vmm.insert_net_device(network_interface).is_ok());

        let mac = MacAddr::parse_str("01:23:45:67:89:0A").unwrap();
        // test update network interface
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif"),
            host_dev_name: String::from("hostname2"),
            guest_mac: Some(mac),
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };
        assert!(vmm.insert_net_device(network_interface).is_ok());

        // Test insert new net device with same mac fails.
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif2"),
            host_dev_name: String::from("hostname3"),
            guest_mac: Some(mac),
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };
        assert!(vmm.insert_net_device(network_interface).is_err());

        // Test that update post-boot fails.
        vmm.set_instance_state(InstanceState::Running);
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif"),
            host_dev_name: String::from("hostname2"),
            guest_mac: None,
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };
        assert!(vmm.insert_net_device(network_interface).is_err());
    }

    #[test]
    fn test_update_net_device() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        let tbc_1mtps = TokenBucketConfig {
            size: 1024 * 1024,
            one_time_burst: None,
            refill_time: 1000,
        };
        let tbc_2mtps = TokenBucketConfig {
            size: 2 * 1024 * 1024,
            one_time_burst: None,
            refill_time: 1000,
        };

        vmm.insert_net_device(NetworkInterfaceConfig {
            iface_id: String::from("1"),
            host_dev_name: String::from("hostname4"),
            guest_mac: None,
            rx_rate_limiter: Some(RateLimiterConfig {
                bandwidth: Some(tbc_1mtps),
                ops: None,
            }),
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        })
        .unwrap();

        vmm.update_net_rate_limiters(NetworkInterfaceUpdateConfig {
            iface_id: "1".to_string(),
            rx_rate_limiter: Some(RateLimiterConfig {
                bandwidth: None,
                ops: Some(tbc_2mtps),
            }),
            tx_rate_limiter: Some(RateLimiterConfig {
                bandwidth: None,
                ops: Some(tbc_2mtps),
            }),
        })
        .unwrap();

        {
            let nic_1: &mut NetworkInterfaceConfig = vmm
                .device_configs
                .network_interface
                .iter_mut()
                .next()
                .unwrap();
            // The RX bandwidth should be unaffected.
            assert_eq!(nic_1.rx_rate_limiter.unwrap().bandwidth.unwrap(), tbc_1mtps);
            // The RX ops should be set to 2mtps.
            assert_eq!(nic_1.rx_rate_limiter.unwrap().ops.unwrap(), tbc_2mtps);
            // The TX bandwith should be unlimited (unaffected).
            assert_eq!(nic_1.tx_rate_limiter.unwrap().bandwidth, None);
            // The TX ops should be set to 2mtps.
            assert_eq!(nic_1.tx_rate_limiter.unwrap().ops.unwrap(), tbc_2mtps);
        }

        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.setup_interrupt_controller().is_ok());
        vmm.default_kernel_config(None);
        vmm.init_mmio_device_manager();

        vmm.attach_net_devices().unwrap();
        vmm.set_instance_state(InstanceState::Running);

        // The update should fail before device activation.
        assert!(vmm
            .update_net_rate_limiters(NetworkInterfaceUpdateConfig {
                iface_id: "1".to_string(),
                rx_rate_limiter: None,
                tx_rate_limiter: None,
            })
            .is_err());

        // Activate the device
        {
            let device_manager = vmm.mmio_device_manager.as_ref().unwrap();
            let bus_device_mutex = device_manager
                .get_device(DeviceType::Virtio(TYPE_NET), "1")
                .unwrap();
            let bus_device = &mut *bus_device_mutex.lock().unwrap();
            let mmio_device: &mut MmioTransport = bus_device
                .as_mut_any()
                .downcast_mut::<MmioTransport>()
                .unwrap();

            assert!(mmio_device
                .device_mut()
                .activate(
                    vmm.vm.memory().unwrap().clone(),
                    EventFd::new(libc::EFD_NONBLOCK).unwrap(),
                    Arc::new(AtomicUsize::new(0)),
                    vec![Queue::new(0), Queue::new(0)],
                    vec![
                        EventFd::new(libc::EFD_NONBLOCK).unwrap(),
                        EventFd::new(libc::EFD_NONBLOCK).unwrap()
                    ],
                )
                .is_ok());
        }

        // the update should succeed after the device activation
        vmm.update_net_rate_limiters(NetworkInterfaceUpdateConfig {
            iface_id: "1".to_string(),
            rx_rate_limiter: Some(RateLimiterConfig {
                bandwidth: Some(tbc_2mtps),
                ops: None,
            }),
            tx_rate_limiter: Some(RateLimiterConfig {
                bandwidth: Some(tbc_1mtps),
                ops: None,
            }),
        })
        .unwrap();
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_machine_configuration() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        // test the default values of machine config
        // vcpu_count = 1
        assert_eq!(vmm.vm_config.vcpu_count, Some(1));
        // mem_size = 128
        assert_eq!(vmm.vm_config.mem_size_mib, Some(128));
        // ht_enabled = false
        assert_eq!(vmm.vm_config.ht_enabled, Some(false));
        // no cpu template
        assert!(vmm.vm_config.cpu_template.is_none());

        // 1. Tests with no hyperthreading
        // test put machine configuration for vcpu count with valid value
        let machine_config = VmConfig {
            vcpu_count: Some(3),
            mem_size_mib: None,
            ht_enabled: None,
            cpu_template: None,
        };
        assert!(vmm.set_vm_configuration(machine_config).is_ok());
        assert_eq!(vmm.vm_config.vcpu_count, Some(3));
        assert_eq!(vmm.vm_config.mem_size_mib, Some(128));
        assert_eq!(vmm.vm_config.ht_enabled, Some(false));

        // test put machine configuration for mem size with valid value
        let machine_config = VmConfig {
            vcpu_count: None,
            mem_size_mib: Some(256),
            ht_enabled: None,
            cpu_template: None,
        };
        assert!(vmm.set_vm_configuration(machine_config).is_ok());
        assert_eq!(vmm.vm_config.vcpu_count, Some(3));
        assert_eq!(vmm.vm_config.mem_size_mib, Some(256));
        assert_eq!(vmm.vm_config.ht_enabled, Some(false));

        // Test Error cases for put_machine_configuration with invalid value for vcpu_count
        // Test that the put method return error & that the vcpu value is not changed
        let machine_config = VmConfig {
            vcpu_count: Some(0),
            mem_size_mib: None,
            ht_enabled: None,
            cpu_template: None,
        };
        assert!(vmm.set_vm_configuration(machine_config).is_err());
        assert_eq!(vmm.vm_config.vcpu_count, Some(3));

        // Test Error cases for put_machine_configuration with invalid value for the mem_size_mib
        // Test that the put method return error & that the mem_size_mib value is not changed
        let machine_config = VmConfig {
            vcpu_count: Some(1),
            mem_size_mib: Some(0),
            ht_enabled: Some(false),
            cpu_template: Some(CpuFeaturesTemplate::T2),
        };
        assert!(vmm.set_vm_configuration(machine_config).is_err());
        assert_eq!(vmm.vm_config.vcpu_count, Some(3));
        assert_eq!(vmm.vm_config.mem_size_mib, Some(256));
        assert_eq!(vmm.vm_config.ht_enabled, Some(false));
        assert!(vmm.vm_config.cpu_template.is_none());

        // 2. Test with hyperthreading enabled
        // Test that you can't change the hyperthreading value to false when the vcpu count
        // is odd
        let machine_config = VmConfig {
            vcpu_count: None,
            mem_size_mib: None,
            ht_enabled: Some(true),
            cpu_template: None,
        };
        assert!(vmm.set_vm_configuration(machine_config).is_err());
        assert_eq!(vmm.vm_config.ht_enabled, Some(false));
        // Test that you can change the ht flag when you have a valid vcpu count
        // Also set the CPU Template since we are here
        let machine_config = VmConfig {
            vcpu_count: Some(2),
            mem_size_mib: None,
            ht_enabled: Some(true),
            cpu_template: Some(CpuFeaturesTemplate::T2),
        };
        assert!(vmm.set_vm_configuration(machine_config).is_ok());
        assert_eq!(vmm.vm_config.vcpu_count, Some(2));
        assert_eq!(vmm.vm_config.ht_enabled, Some(true));
        assert_eq!(vmm.vm_config.cpu_template, Some(CpuFeaturesTemplate::T2));

        // 3. Test update vm configuration after boot.
        vmm.set_instance_state(InstanceState::Running);
        let machine_config = VmConfig {
            vcpu_count: Some(2),
            mem_size_mib: None,
            ht_enabled: Some(true),
            cpu_template: Some(CpuFeaturesTemplate::T2),
        };
        assert!(vmm.set_vm_configuration(machine_config).is_err());
    }

    #[test]
    fn new_epoll_context_test() {
        assert!(EpollContext::new().is_ok());
    }

    #[test]
    fn add_epollin_event_test() {
        let mut ep = EpollContext::new().unwrap();
        let evfd = EventFd::new(libc::EFD_NONBLOCK).unwrap();

        // adding new event should work
        assert!(ep.add_epollin_event(&evfd, EpollDispatch::Exit).is_ok());
    }

    #[test]
    fn epoll_event_test() {
        let mut ep = EpollContext::new().unwrap();
        let evfd = EventFd::new(libc::EFD_NONBLOCK).unwrap();

        // adding new event should work
        assert!(ep.add_epollin_event(&evfd, EpollDispatch::Exit).is_ok());

        let evpoll_events_len = 10;
        let mut events = vec![epoll::Event::new(epoll::Events::empty(), 0); evpoll_events_len];

        // epoll should have no pending events
        let epollret = epoll::wait(ep.epoll_raw_fd, 0, &mut events[..]);
        let num_events = epollret.unwrap();
        assert_eq!(num_events, 0);

        // raise the event
        assert!(evfd.write(1).is_ok());

        // epoll should report one event
        let epollret = epoll::wait(ep.epoll_raw_fd, 0, &mut events[..]);
        let num_events = epollret.unwrap();
        assert_eq!(num_events, 1);

        // reported event should be the one we raised
        let idx = events[0].data as usize;
        assert!(ep.dispatch_table[idx].is_some());
        assert_eq!(
            *ep.dispatch_table[idx].as_ref().unwrap(),
            EpollDispatch::Exit
        );
    }

    #[test]
    fn test_check_health() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm.check_health().is_err());

        vmm.set_kernel_config(KernelConfig {
            cmdline: kernel_cmdline::Cmdline::new(10),
            kernel_file: tempfile::tempfile().unwrap(),
        });
        assert!(vmm.check_health().is_ok());
    }

    #[test]
    fn test_is_instance_initialized() {
        let vmm = create_vmm_object(InstanceState::Uninitialized);
        assert_eq!(vmm.is_instance_initialized(), false);

        let vmm = create_vmm_object(InstanceState::Starting);
        assert_eq!(vmm.is_instance_initialized(), true);

        let vmm = create_vmm_object(InstanceState::Running);
        assert_eq!(vmm.is_instance_initialized(), true);
    }

    #[test]
    fn test_epoll_stdin_event() {
        let mut epoll_context = EpollContext::new().unwrap();

        // If this unit test is run without a terminal attached (i.e ssh without pseudo terminal,
        // request, jailer with `--daemonize` flag on) EPOLL_CTL_ADD would return EPERM
        // on STDIN_FILENO. So, we are using `isatty` to check whether STDIN_FILENO refers
        // to a terminal. If it does not, we are no longer asserting against
        // `epoll_context.dispatch_table[epoll_context.stdin_index as usize]` holding any value.
        if unsafe { libc::isatty(libc::STDIN_FILENO as i32) } == 1 {
            epoll_context.enable_stdin_event();
            assert_eq!(
                epoll_context.dispatch_table[epoll_context.stdin_index as usize].unwrap(),
                EpollDispatch::Stdin
            );
        }

        epoll_context.enable_stdin_event();
        epoll_context.disable_stdin_event();
        assert!(epoll_context.dispatch_table[epoll_context.stdin_index as usize].is_none());
    }

    #[test]
    fn test_attach_net_devices() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());

        vmm.default_kernel_config(None);
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");
        vmm.init_mmio_device_manager();

        // test create network interface
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif"),
            host_dev_name: String::from("hostname5"),
            guest_mac: None,
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };

        assert!(vmm.insert_net_device(network_interface).is_ok());

        assert!(vmm.attach_net_devices().is_ok());
        // a second call to attach_net_devices should fail because when
        // we are creating the virtio::Net object, we are taking the tap.
        assert!(vmm.attach_net_devices().is_err());
    }

    #[test]
    fn test_init_devices() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.default_kernel_config(None);
        assert!(vmm.init_guest_memory().is_ok());
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");

        vmm.init_mmio_device_manager();
        assert!(vmm.attach_virtio_devices().is_ok());
    }

    #[test]
    fn test_configure_boot_source() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        // Test invalid kernel path.
        assert!(vmm
            .configure_boot_source(BootSourceConfig {
                kernel_image_path: String::from("dummy-path"),
                boot_args: None
            })
            .is_err());

        // Test valid kernel path and invalid cmdline.
        let kernel_file = NamedTempFile::new().expect("Failed to create temporary kernel file.");
        let kernel_path = String::from(kernel_file.path().to_path_buf().to_str().unwrap());
        let invalid_cmdline = String::from_utf8(vec![b'X'; arch::CMDLINE_MAX_SIZE + 1]).unwrap();
        assert!(vmm
            .configure_boot_source(BootSourceConfig {
                kernel_image_path: kernel_path.clone(),
                boot_args: Some(invalid_cmdline)
            })
            .is_err());

        // Test valid configuration.
        assert!(vmm
            .configure_boot_source(BootSourceConfig {
                kernel_image_path: kernel_path.clone(),
                boot_args: None
            })
            .is_ok());
        assert!(vmm
            .configure_boot_source(BootSourceConfig {
                kernel_image_path: kernel_path.clone(),
                boot_args: Some(String::from("reboot=k"))
            })
            .is_ok());

        // Test valid configuration after boot (should fail).
        vmm.set_instance_state(InstanceState::Running);
        assert!(vmm
            .configure_boot_source(BootSourceConfig {
                kernel_image_path: kernel_path.clone(),
                boot_args: None
            })
            .is_err());
    }

    #[test]
    fn test_block_device_rescan() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.default_kernel_config(None);

        let root_file = NamedTempFile::new().unwrap();
        let scratch_file = NamedTempFile::new().unwrap();
        let scratch_id = "not_root".to_string();

        let root_block_device = BlockDeviceConfig {
            drive_id: String::from("root"),
            path_on_host: root_file.path().to_path_buf(),
            is_root_device: true,
            partuuid: None,
            is_read_only: false,
            rate_limiter: None,
        };
        let non_root_block_device = BlockDeviceConfig {
            drive_id: scratch_id.clone(),
            path_on_host: scratch_file.path().to_path_buf(),
            is_root_device: false,
            partuuid: None,
            is_read_only: true,
            rate_limiter: None,
        };

        assert!(vmm.insert_block_device(root_block_device.clone()).is_ok());
        assert!(vmm
            .insert_block_device(non_root_block_device.clone())
            .is_ok());

        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());
        assert!(vmm.setup_interrupt_controller().is_ok());

        vmm.init_mmio_device_manager();

        {
            let dummy_box = Box::new(DummyDevice { dummy: 0 });
            let device_manager = vmm.mmio_device_manager.as_mut().unwrap();

            // Use a dummy command line as it is not used in this test.
            let _addr = device_manager
                .register_virtio_device(
                    vmm.vm.fd(),
                    dummy_box,
                    &mut kernel_cmdline::Cmdline::new(arch::CMDLINE_MAX_SIZE),
                    TYPE_BLOCK,
                    &scratch_id,
                )
                .unwrap();
        }

        vmm.set_instance_state(InstanceState::Running);

        // Test valid rescan_block_device.
        assert!(vmm.rescan_block_device(&scratch_id).is_ok());

        // Test rescan block device with size not a multiple of sector size.
        let new_size = 10 * virtio::block::SECTOR_SIZE + 1;
        scratch_file.as_file().set_len(new_size).unwrap();
        assert!(vmm.rescan_block_device(&scratch_id).is_ok());

        // Test rescan block device with invalid path.
        let prev_path = non_root_block_device.path_on_host().clone();
        vmm.update_block_device_path(&scratch_id, PathBuf::from("foo"));
        assert_match!(
            vmm.rescan_block_device(&scratch_id),
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::BlockDeviceUpdateFailed,
            ))
        );
        vmm.update_block_device_path(&scratch_id, prev_path);

        // Test rescan_block_device with invalid ID.
        assert_match!(
            vmm.rescan_block_device(&"foo".to_string()),
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::InvalidBlockDeviceID
            ))
        );

        vmm.change_id(&scratch_id, "scratch");
        assert_match!(
            vmm.rescan_block_device(&scratch_id),
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::InvalidBlockDeviceID
            ))
        );

        // Test rescan_block_device with invalid device address.
        vmm.remove_device_info(TYPE_BLOCK, &scratch_id);

        assert_match!(
            vmm.rescan_block_device(&scratch_id),
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::InvalidBlockDeviceID
            ))
        );

        // Test rescan not allowed.
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm
            .insert_block_device(non_root_block_device.clone())
            .is_ok());

        assert_match!(
            vmm.rescan_block_device(&scratch_id),
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::OperationNotAllowedPreBoot,
            ))
        );
    }

    #[test]
    fn test_init_logger() {
        // Error case: update after instance is running
        let log_file = NamedTempFile::new().unwrap();
        let metrics_file = NamedTempFile::new().unwrap();
        let desc = LoggerConfig {
            log_fifo: log_file.path().to_str().unwrap().to_string(),
            metrics_fifo: metrics_file.path().to_str().unwrap().to_string(),
            level: LoggerLevel::Warning,
            show_level: true,
            show_log_origin: true,
            #[cfg(target_arch = "x86_64")]
            options: vec![],
        };

        let mut vmm = create_vmm_object(InstanceState::Running);
        assert!(vmm.init_logger(desc).is_err());

        // Reset vmm state to test the other scenarios.
        vmm.set_instance_state(InstanceState::Uninitialized);

        // Error case: initializing logger with invalid log pipe returns error.
        let desc = LoggerConfig {
            log_fifo: String::from("not_found_file_log"),
            metrics_fifo: metrics_file.path().to_str().unwrap().to_string(),
            level: LoggerLevel::Debug,
            show_level: false,
            show_log_origin: false,
            #[cfg(target_arch = "x86_64")]
            options: vec![],
        };
        assert!(vmm.init_logger(desc).is_err());

        // Error case: initializing logger with invalid metrics pipe returns error.
        let desc = LoggerConfig {
            log_fifo: log_file.path().to_str().unwrap().to_string(),
            metrics_fifo: String::from("not_found_file_metrics"),
            level: LoggerLevel::Debug,
            show_level: false,
            show_log_origin: false,
            #[cfg(target_arch = "x86_64")]
            options: vec![],
        };
        assert!(vmm.init_logger(desc).is_err());

        // Error case: initializing logger with invalid metrics and log pipe returns error.
        let desc = LoggerConfig {
            log_fifo: String::from("not_found_file_log"),
            metrics_fifo: String::from("not_found_file_metrics"),
            level: LoggerLevel::Warning,
            show_level: false,
            show_log_origin: false,
            #[cfg(target_arch = "x86_64")]
            options: vec![],
        };
        assert!(vmm.init_logger(desc).is_err());

        // Initializing logger with valid pipes is ok.
        let log_file = NamedTempFile::new().unwrap();
        let metrics_file = NamedTempFile::new().unwrap();
        let desc = LoggerConfig {
            log_fifo: log_file.path().to_str().unwrap().to_string(),
            metrics_fifo: metrics_file.path().to_str().unwrap().to_string(),
            level: LoggerLevel::Info,
            show_level: true,
            show_log_origin: true,
            #[cfg(target_arch = "x86_64")]
            options: vec![LogOption::LogDirtyPages],
        };
        // Flushing metrics before initializing logger is not erroneous.
        assert!(vmm.flush_metrics().is_ok());

        assert!(vmm.init_logger(desc.clone()).is_ok());
        assert!(vmm.init_logger(desc).is_err());

        assert!(vmm.flush_metrics().is_ok());

        let f = File::open(metrics_file).unwrap();
        let mut reader = BufReader::new(f);

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("utc_timestamp_ms"));

        // It is safe to do that because the tests are run sequentially (so no other test may be
        // writing to the same file.
        assert!(vmm.flush_metrics().is_ok());
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("utc_timestamp_ms"));

        // Validate logfile works.
        warn!("this is a test");

        let f = File::open(log_file).unwrap();
        let mut reader = BufReader::new(f);

        let mut line = String::new();
        loop {
            if line.contains("this is a test") {
                break;
            }
            if reader.read_line(&mut line).unwrap() == 0 {
                // If it ever gets here, this assert will fail.
                assert!(line.contains("this is a test"));
            }
        }

        // Validate logging the boot time works.
        Vmm::log_boot_time(&TimestampUs::default());
        let mut line = String::new();
        loop {
            if line.contains("Guest-boot-time =") {
                break;
            }
            if reader.read_line(&mut line).unwrap() == 0 {
                // If it ever gets here, this assert will fail.
                assert!(line.contains("Guest-boot-time ="));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_dirty_page_count() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert_eq!(vmm.get_dirty_page_count(), 0);
        // Booting an actual guest and getting real data is covered by `kvm::tests::run_code_test`.
    }

    #[test]
    fn test_create_vcpus() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.default_kernel_config(None);

        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());

        #[cfg(target_arch = "x86_64")]
        // `KVM_CREATE_VCPU` fails if the irqchip is not created beforehand. This is x86_64 speciifc.
        vmm.vm
            .setup_irqchip()
            .expect("Cannot create IRQCHIP or PIT");

        assert!(vmm
            .create_vcpus(GuestAddress(0x0), TimestampUs::default())
            .is_ok());
    }

    #[test]
    #[should_panic(expected = "Cannot load kernel prior allocating memory!")]
    fn test_load_kernel_no_mem() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.load_kernel().unwrap();
    }

    #[test]
    fn test_load_kernel() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());

        assert_eq!(
            vmm.load_kernel().unwrap_err().to_string(),
            "Cannot start microvm without kernel configuration."
        );

        vmm.default_kernel_config(None);

        #[cfg(target_arch = "aarch64")]
        assert_eq!(
            vmm.load_kernel().unwrap_err().to_string(),
            "Cannot load kernel due to invalid memory configuration or invalid kernel image. Failed to read magic number"
        );

        #[cfg(target_arch = "x86_64")]
        assert_eq!(
            vmm.load_kernel().unwrap_err().to_string(),
            "Cannot load kernel due to invalid memory configuration or invalid kernel image. Failed to read ELF header"
        );

        vmm.default_kernel_config(Some(good_kernel_file()));
        assert!(vmm.load_kernel().is_ok());
    }

    #[test]
    #[should_panic(expected = "Cannot configure registers prior to allocating memory!")]
    fn test_configure_system_no_mem() {
        let vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.configure_system(&Vec::new()).unwrap();
    }

    #[test]
    fn test_configure_system() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        assert!(vmm.init_guest_memory().is_ok());

        // test that we can do this any number of times we want.
        assert!(vmm.init_guest_memory().is_ok());

        assert!(vmm.vm.memory().is_some());

        assert_eq!(
            vmm.configure_system(&Vec::new()).unwrap_err().to_string(),
            "Cannot start microvm without kernel configuration."
        );

        vmm.default_kernel_config(None);

        // We need this so that configure_system finds a properly setup GIC device
        #[cfg(target_arch = "aarch64")]
        assert!(vmm.vm.setup_irqchip(1).is_ok());
        assert!(vmm.configure_system(&Vec::new()).is_ok());

        vmm.stdin_handle.lock().set_canon_mode().unwrap();
    }

    #[test]
    fn test_attach_virtio_devices() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.default_kernel_config(None);

        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");

        // Create test network interface.
        let network_interface = NetworkInterfaceConfig {
            iface_id: String::from("netif"),
            host_dev_name: String::from("hostname6"),
            guest_mac: None,
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };

        assert!(vmm.insert_net_device(network_interface).is_ok());
        assert!(vmm.attach_virtio_devices().is_ok());
        assert!(vmm.mmio_device_manager.is_some());
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_attach_legacy_devices() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");

        assert!(vmm.attach_legacy_devices().is_ok());
        assert!(vmm.pio_device_manager.io_bus.get_device(0x3f8).is_some());
        assert!(vmm.pio_device_manager.io_bus.get_device(0x2f8).is_some());
        assert!(vmm.pio_device_manager.io_bus.get_device(0x3e8).is_some());
        assert!(vmm.pio_device_manager.io_bus.get_device(0x2e8).is_some());
        assert!(vmm.pio_device_manager.io_bus.get_device(0x060).is_some());
        assert!(vmm.configure_stdin().is_ok());
        assert!(vmm.handle_stdin_event(&[b'a', b'b', b'c']).is_ok());
        vmm.stdin_handle.lock().set_canon_mode().unwrap();
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_attach_legacy_devices_without_uart() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());

        let guest_mem = vmm.vm.memory().unwrap().clone();
        let device_manager = MMIODeviceManager::new(
            guest_mem,
            &mut (arch::MMIO_MEM_START),
            (arch::IRQ_BASE, arch::IRQ_MAX),
        );
        vmm.mmio_device_manager = Some(device_manager);

        vmm.default_kernel_config(None);
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");
        assert!(vmm.attach_legacy_devices().is_ok());

        let dev_man = vmm.mmio_device_manager.as_ref().unwrap();
        // On aarch64, we are using first region of the memory
        // reserved for attaching MMIO devices for measuring boot time.
        assert!(dev_man.bus.get_device(arch::MMIO_MEM_START).is_none());
        assert!(dev_man
            .get_device_info()
            .get(&(DeviceType::Serial, DeviceType::Serial.to_string()))
            .is_none());
        assert!(dev_man
            .get_device_info()
            .get(&(DeviceType::RTC, "rtc".to_string()))
            .is_some());
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_attach_legacy_devices_with_uart() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);
        assert!(vmm.init_guest_memory().is_ok());
        assert!(vmm.vm.memory().is_some());

        let guest_mem = vmm.vm.memory().unwrap().clone();
        let device_manager = MMIODeviceManager::new(
            guest_mem,
            &mut (arch::MMIO_MEM_START),
            (arch::IRQ_BASE, arch::IRQ_MAX),
        );
        vmm.mmio_device_manager = Some(device_manager);

        vmm.default_kernel_config(None);
        vmm.setup_interrupt_controller()
            .expect("Failed to setup interrupt controller");
        {
            let kernel_config = vmm.kernel_config.as_mut().unwrap();
            kernel_config.cmdline.insert("console", "tty1").unwrap();
        }
        assert!(vmm.attach_legacy_devices().is_ok());
        let dev_man = vmm.mmio_device_manager.as_ref().unwrap();
        assert!(dev_man
            .get_device_info()
            .get(&(DeviceType::Serial, DeviceType::Serial.to_string()))
            .is_some());

        let serial_device = vmm.get_serial_device();
        assert!(serial_device.is_some());

        assert!(vmm.configure_stdin().is_ok());
        vmm.stdin_handle.lock().set_canon_mode().unwrap();
        assert!(vmm.handle_stdin_event(&[b'a', b'b', b'c']).is_ok());
    }

    #[test]
    fn test_configure_vmm_from_json() {
        let mut vmm = create_vmm_object(InstanceState::Uninitialized);

        let kernel_file = NamedTempFile::new().unwrap();
        let rootfs_file = NamedTempFile::new().unwrap();

        // We will test different scenarios with invalid resources configuration and
        // check the expected errors. We include configuration for the kernel and rootfs
        // in every json because they are mandatory fields. If we don't configure
        // these resources, it is considered an invalid json and the test will crash.

        // Invalid kernel path.
        let mut json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "/invalid/path",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ]
            }}"#,
            rootfs_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::BootSource(
                ErrorKind::User,
                BootSourceConfigError::InvalidKernelPath(_),
            )) => (),
            _ => unreachable!(),
        }

        // Invalid rootfs path.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "/invalid/path",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ]
            }}"#,
            kernel_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::DriveConfig(
                ErrorKind::User,
                DriveError::InvalidBlockDevicePath,
            )) => (),
            _ => unreachable!(),
        }

        // Invalid vCPU number.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ],
                    "machine-config": {{
                        "vcpu_count": 0,
                        "mem_size_mib": 1024,
                        "ht_enabled": false
                    }}
            }}"#,
            kernel_file.path().to_str().unwrap(),
            rootfs_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::MachineConfig(
                ErrorKind::User,
                VmConfigError::InvalidVcpuCount,
            )) => (),
            _ => unreachable!(),
        }

        // Invalid memory size.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ],
                    "machine-config": {{
                        "vcpu_count": 2,
                        "mem_size_mib": 0,
                        "ht_enabled": false
                    }}
            }}"#,
            kernel_file.path().to_str().unwrap(),
            rootfs_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::MachineConfig(
                ErrorKind::User,
                VmConfigError::InvalidMemorySize,
            )) => (),
            _ => unreachable!(),
        }

        // Invalid path for logger pipe.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ],
                    "logger": {{
                        "log_fifo": "/invalid/path",
                        "metrics_fifo": "metrics.fifo"
                    }}
            }}"#,
            kernel_file.path().to_str().unwrap(),
            rootfs_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::Logger(
                ErrorKind::User,
                LoggerConfigError::InitializationFailure { .. },
            )) => (),
            _ => unreachable!(),
        }

        // Reuse of a host name.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ],
                    "network-interfaces": [
                        {{
                            "iface_id": "netif1",
                            "host_dev_name": "hostname7"
                        }},
                        {{
                            "iface_id": "netif2",
                            "host_dev_name": "hostname7"
                        }}
                    ]
            }}"#,
            kernel_file.path().to_str().unwrap(),
            rootfs_file.path().to_str().unwrap()
        );

        match vmm.configure_from_json(json) {
            Err(VmmActionError::NetworkConfig(
                ErrorKind::User,
                NetworkInterfaceError::HostDeviceNameInUse { .. },
            )) => (),
            _ => unreachable!(),
        }

        // Let's try now passing a valid configuration. We won't include any logger
        // configuration because the logger was already initialized in another test
        // of this module and the reinitialization of it will cause crashing.
        json = format!(
            r#"{{
                    "boot-source": {{
                        "kernel_image_path": "{}",
                        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
                    }},
                    "drives": [
                        {{
                            "drive_id": "rootfs",
                            "path_on_host": "{}",
                            "is_root_device": true,
                            "is_read_only": false
                        }}
                    ],
                    "network-interfaces": [
                        {{
                            "iface_id": "netif",
                            "host_dev_name": "hostname8"
                        }}
                    ],
                     "machine-config": {{
                            "vcpu_count": 2,
                            "mem_size_mib": 1024,
                            "ht_enabled": false
                     }}
            }}"#,
            kernel_file.path().to_str().unwrap(),
            rootfs_file.path().to_str().unwrap()
        );

        assert!(vmm.configure_from_json(json).is_ok());
    }
}
*/
