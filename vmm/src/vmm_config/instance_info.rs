// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std;
use std::fmt::{Display, Formatter, Result};

use device_manager;
use devices;
use kernel::loader as kernel_loader;
use memory_model::GuestMemoryError;
use seccomp;
use vstate;

/// The microvm state. When Firecracker starts, the instance state is Uninitialized.
/// Once start_microvm method is called, the state goes from Uninitialized to Starting.
/// The state is changed to Running before ending the start_microvm method.
/// Halting and Halted are currently unsupported.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum InstanceState {
    /// Microvm is not initialized.
    Uninitialized,
    /// Microvm is starting.
    Starting,
    /// Microvm is running.
    Running,
    /// Microvm received a halt instruction.
    Halting,
    /// Microvm is halted.
    Halted,
}

/// The strongly typed that contains general information about the microVM.
#[derive(Debug, Serialize)]
pub struct InstanceInfo {
    /// The ID of the microVM.
    pub id: String,
    /// The state of the microVM.
    pub state: InstanceState,
    /// The version of the VMM that runs the microVM.
    pub vmm_version: String,
}

/// Errors associated with starting the instance.
// TODO: add error kind to these variants because not all these errors are user or internal.
pub enum StartMicrovmError {
    /// This error is thrown by the minimal boot loader implementation.
    /// It is related to a faulty memory configuration.
    ConfigureSystem(arch::Error),
    /// Cannot configure the VM.
    ConfigureVm(vstate::Error),
    /// Unable to seek the block device backing file due to invalid permissions or
    /// the file was deleted/corrupted.
    CreateBlockDevice(std::io::Error),
    /// Split this at some point.
    /// Internal errors are due to resource exhaustion.
    /// Users errors are due to invalid permissions.
    CreateNetDevice(devices::virtio::Error),
    /// Failed to create a `RateLimiter` object.
    CreateRateLimiter(std::io::Error),
    /// Failed to create the vsock device.
    CreateVsockDevice,
    /// The device manager was not configured.
    DeviceManager,
    /// Cannot read from an Event file descriptor.
    EventFd,
    /// Memory regions are overlapping or mmap fails.
    GuestMemory(GuestMemoryError),
    /// The kernel command line is invalid.
    KernelCmdline(String),
    /// Cannot load kernel due to invalid memory configuration or invalid kernel image.
    KernelLoader(kernel_loader::Error),
    /// Cannot add devices to the Legacy I/O Bus.
    LegacyIOBus(device_manager::legacy::Error),
    /// Cannot load command line string.
    LoadCommandline(kernel::cmdline::Error),
    /// The start command was issued more than once.
    MicroVMAlreadyRunning,
    /// Cannot start the VM because the kernel was not configured.
    MissingKernelConfig,
    /// The net device configuration is missing the tap device.
    NetDeviceNotConfigured,
    /// Cannot open the block device backing file.
    OpenBlockDevice(std::io::Error),
    /// Cannot initialize a MMIO Block Device or add a device to the MMIO Bus.
    RegisterBlockDevice(device_manager::mmio::Error),
    /// Cannot add event to Epoll.
    RegisterEvent,
    /// Cannot add a device to the MMIO Bus.
    RegisterMMIODevice(device_manager::mmio::Error),
    /// Cannot initialize a MMIO Network Device or add a device to the MMIO Bus.
    RegisterNetDevice(device_manager::mmio::Error),
    /// Cannot initialize a MMIO Vsock Device or add a device to the MMIO Bus.
    RegisterVsockDevice(device_manager::mmio::Error),
    /// Cannot build seccomp filters.
    SeccompFilters(seccomp::Error),
    /// Cannot create a new vCPU file descriptor.
    Vcpu(vstate::Error),
    /// vCPU configuration failed.
    VcpuConfigure(vstate::Error),
    /// vCPUs were not configured.
    VcpusNotConfigured,
    /// Cannot spawn a new vCPU thread.
    VcpuSpawn(std::io::Error),
    /// Cannot set mode for terminal.
    StdinHandle(std::io::Error),
}

/// It's convenient to automatically convert `kernel::cmdline::Error`s
/// to `StartMicrovmError`s.
impl std::convert::From<kernel::cmdline::Error> for StartMicrovmError {
    fn from(e: kernel::cmdline::Error) -> StartMicrovmError {
        StartMicrovmError::KernelCmdline(e.to_string())
    }
}

impl Display for StartMicrovmError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::StartMicrovmError::*;
        match *self {
            ConfigureSystem(ref err) => write!(f, "Faulty memory configuration: {}", err),
            ConfigureVm(ref err) => write!(f, "Cannot configure virtual machine: {}", err),
            CreateBlockDevice(ref err) => write!(
                f,
                "Unable to seek the block device backing file due to invalid permissions or \
                 the file was deleted/corrupted. Error number: {}",
                err
            ),
            CreateRateLimiter(ref err) => write!(f, "Cannot create RateLimiter: {}", err),
            CreateVsockDevice => write!(f, "Cannot create vsock device."),
            CreateNetDevice(ref err) => write!(f, "Cannot create network device: {}", err),
            DeviceManager => write!(f, "The device manager was not configured."),
            EventFd => write!(f, "Cannot read from an Event file descriptor."),
            GuestMemory(ref err) => write!(f, "Invalid memory configuration: {}", err),
            KernelCmdline(ref err) => write!(f, "Invalid kernel command line: {}", err),
            KernelLoader(ref err) => write!(
                f,
                "Cannot load kernel due to invalid memory configuration or invalid kernel \
                 image: {}",
                err
            ),
            LegacyIOBus(ref err) => write!(f, "Cannot add devices to the legacy I/O Bus: {}", err),
            LoadCommandline(ref err) => write!(f, "Cannot load command line string: {}", err),
            MicroVMAlreadyRunning => write!(f, "Microvm already running."),
            MissingKernelConfig => write!(f, "Cannot start microvm without kernel configuration."),
            NetDeviceNotConfigured => {
                write!(f, "The net device configuration is missing the tap device.")
            }
            OpenBlockDevice(ref err) => {
                write!(f, "Cannot open the block device backing file: {}", err)
            }
            RegisterBlockDevice(ref err) => write!(
                f,
                "Cannot initialize a MMIO Block Device or add a device to the MMIO Bus: {}",
                err
            ),
            RegisterEvent => write!(f, "Cannot add event to Epoll."),
            RegisterMMIODevice(ref err) => {
                write!(f, "Cannot add a device to the MMIO Bus: {}", err)
            }
            RegisterNetDevice(ref err) => write!(
                f,
                "Cannot initialize a MMIO Network Device or add a device to the MMIO Bus: {}",
                err
            ),
            RegisterVsockDevice(ref err) => write!(
                f,
                "Cannot initialize a MMIO Vsock Device or add a device to the MMIO Bus: {}",
                err
            ),
            SeccompFilters(ref err) => write!(f, "Cannot build seccomp filters: {}", err),
            Vcpu(ref err) => write!(f, "Cannot create a new vCPU: {}", err),
            VcpuConfigure(ref err) => write!(f, "vCPU configuration failed: {}", err),
            VcpusNotConfigured => write!(f, "vCPUs were not configured."),
            StdinHandle(ref err) => write!(f, "Failed to set mode for terminal: {}", err),
            VcpuSpawn(ref err) => write!(f, "Cannot spawn vCPU thread: {}", err),
        }
    }
}
