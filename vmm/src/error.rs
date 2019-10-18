// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Contains error related logic.

use std::fmt::{Display, Formatter};
use std::io;

use super::{
    device_manager, vmm_config::boot_source::BootSourceConfigError, vmm_config::drive::DriveError,
    vmm_config::logger::LoggerConfigError, vmm_config::machine_config::VmConfigError,
    vmm_config::net::NetworkInterfaceError, vmm_config::vsock::VsockError, vstate,
};
use devices::legacy::I8042DeviceError;
use kernel::loader as kernel_loader;
use memory_model::GuestMemoryError;

/// Errors associated with the VMM internal logic. These errors cannot be generated by direct user
/// input, but can result from bad configuration of the host (for example if Firecracker doesn't
/// have permissions to open the KVM fd).
#[derive(Debug)]
pub enum Error {
    /// Legacy devices work with Event file descriptors and the creation can fail because
    /// of resource exhaustion.
    #[cfg(target_arch = "x86_64")]
    CreateLegacyDevice(device_manager::legacy::Error),
    /// An operation on the epoll instance failed due to resource exhaustion or bad configuration.
    EpollFd(io::Error),
    /// Cannot read from an Event file descriptor.
    EventFd(io::Error),
    /// An event arrived for a device, but the dispatcher can't find the event (epoll) handler.
    DeviceEventHandlerNotFound,
    /// An epoll handler can't be downcasted to the desired type.
    DeviceEventHandlerInvalidDowncast,
    /// Cannot open /dev/kvm. Either the host does not have KVM or Firecracker does not have
    /// permission to open the file descriptor.
    KvmContext(vstate::Error),
    /// Epoll wait failed.
    Poll(io::Error),
    /// Write to the serial console failed.
    Serial(io::Error),
    /// Cannot create Timer file descriptor.
    TimerFd(io::Error),
    /// Cannot open the VM file descriptor.
    Vm(vstate::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::Error::*;

        match self {
            #[cfg(target_arch = "x86_64")]
            CreateLegacyDevice(e) => write!(f, "Error creating legacy device: {}", e),
            EpollFd(e) => write!(f, "Epoll fd error: {}", e.to_string()),
            EventFd(e) => write!(f, "Event fd error: {}", e.to_string()),
            DeviceEventHandlerNotFound => write!(
                f,
                "Device event handler not found. This might point to a guest device driver issue."
            ),
            DeviceEventHandlerInvalidDowncast => write!(
                f,
                "Device event handler couldn't be downcasted to expected type."
            ),
            KvmContext(e) => write!(f, "Failed to validate KVM support: {}", e),
            Poll(e) => write!(f, "Epoll wait failed: {}", e.to_string()),
            Serial(e) => write!(f, "Error writing to the serial console: {}", e),
            TimerFd(e) => write!(f, "Error creating timer fd: {}", e.to_string()),
            Vm(e) => write!(f, "Error opening VM fd: {}", e),
        }
    }
}

/// Errors associated with starting the instance.
// TODO: add error kind to these variants because not all these errors are user or internal.
#[derive(Debug)]
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
    /// Failed to create the backend for the vsock device.
    CreateVsockBackend(devices::virtio::vsock::VsockUnixBackendError),
    /// Failed to create the vsock device.
    CreateVsockDevice(devices::virtio::vsock::VsockError),
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
    #[cfg(target_arch = "x86_64")]
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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::StartMicrovmError::*;
        match *self {
            ConfigureSystem(ref err) => write!(f, "Faulty memory configuration: {}", err),
            ConfigureVm(ref err) => write!(f, "Cannot configure virtual machine: {}", err),
            CreateBlockDevice(ref err) => write!(
                f,
                "Unable to seek the block device backing file due to invalid permissions or \
                 the file was deleted/corrupted: {}",
                err
            ),
            CreateRateLimiter(ref err) => write!(f, "Cannot create RateLimiter: {}", err),
            CreateVsockBackend(ref err) => {
                write!(f, "Cannot create backend for vsock device: {}", err)
            }
            CreateVsockDevice(ref err) => write!(f, "Cannot create vsock device: {}", err),
            CreateNetDevice(ref err) => write!(f, "Cannot create network device: {}", err),
            DeviceManager => write!(f, "The device manager was not configured."),
            EventFd => write!(f, "Cannot read from an event file descriptor."),
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
            VcpuSpawn(ref err) => write!(f, "Cannot spawn vCPU thread: {}", err),
            StdinHandle(ref err) => write!(f, "Failed to set mode for terminal: {}", err),
        }
    }
}

/// Types of errors associated with vmm actions.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// User Errors describe bad configuration (user input).
    User,
    /// Internal Errors are unrelated to the user and usually refer to logical errors
    /// or bad management of resources (memory, file descriptors & others).
    Internal,
}

/// Wrapper for all errors associated with VMM actions.
#[derive(Debug)]
pub enum VmmActionError {
    /// The action `ConfigureBootSource` failed either because of bad user input (`ErrorKind::User`)
    /// or an internal error (`ErrorKind::Internal`).
    BootSource(ErrorKind, BootSourceConfigError),
    /// One of the actions `InsertBlockDevice`, `RescanBlockDevice` or `UpdateBlockDevicePath`
    /// failed either because of bad user input (`ErrorKind::User`) or an
    /// internal error (`ErrorKind::Internal`).
    DriveConfig(ErrorKind, DriveError),
    /// The action `ConfigureLogger` failed either because of bad user input (`ErrorKind::User`) or
    /// an internal error (`ErrorKind::Internal`).
    Logger(ErrorKind, LoggerConfigError),
    /// One of the actions `GetVmConfiguration` or `SetVmConfiguration` failed either because of bad
    /// input (`ErrorKind::User`) or an internal error (`ErrorKind::Internal`).
    MachineConfig(ErrorKind, VmConfigError),
    /// The action `InsertNetworkDevice` failed either because of bad user input (`ErrorKind::User`)
    /// or an internal error (`ErrorKind::Internal`).
    NetworkConfig(ErrorKind, NetworkInterfaceError),
    /// The action `StartMicroVm` failed either because of bad user input (`ErrorKind::User`) or
    /// an internal error (`ErrorKind::Internal`).
    StartMicrovm(ErrorKind, StartMicrovmError),
    /// The action `SendCtrlAltDel` failed. Details are provided by the device-specific error
    /// `I8042DeviceError`.
    SendCtrlAltDel(ErrorKind, I8042DeviceError),
    /// The action `set_vsock_device` failed either because of bad user input (`ErrorKind::User`)
    /// or an internal error (`ErrorKind::Internal`).
    VsockConfig(ErrorKind, VsockError),
}

// It's convenient to turn DriveErrors into VmmActionErrors directly.
impl std::convert::From<DriveError> for VmmActionError {
    fn from(e: DriveError) -> Self {
        use DriveError::*;

        // This match is used to force developers who add new types of
        // `DriveError`s to explicitly consider what kind they should
        // have. Remove this comment when a match arm that yields
        // something other than `ErrorKind::User` is added.
        let kind = match e {
            // User errors.
            CannotOpenBlockDevice
            | InvalidBlockDeviceID
            | InvalidBlockDevicePath
            | BlockDevicePathAlreadyExists
            | EpollHandlerNotFound
            | BlockDeviceUpdateFailed
            | OperationNotAllowedPreBoot
            | UpdateNotAllowedPostBoot
            | RootBlockDeviceAlreadyAdded => ErrorKind::User,
        };

        VmmActionError::DriveConfig(kind, e)
    }
}

// It's convenient to turn VmConfigErrors into VmmActionErrors directly.
impl std::convert::From<VmConfigError> for VmmActionError {
    fn from(e: VmConfigError) -> Self {
        use VmConfigError::*;

        // This match is used to force developers who add new types of
        // `VmConfigError`s to explicitly consider what kind they should
        // have. Remove this comment when a match arm that yields
        // something other than `ErrorKind::User` is added.
        let kind = match e {
            // User errors.
            InvalidVcpuCount | InvalidMemorySize | UpdateNotAllowedPostBoot => ErrorKind::User,
        };

        VmmActionError::MachineConfig(kind, e)
    }
}

// It's convenient to turn NetworkInterfaceErrors into VmmActionErrors directly.
impl std::convert::From<NetworkInterfaceError> for VmmActionError {
    fn from(e: NetworkInterfaceError) -> Self {
        use NetworkInterfaceError::*;
        use TapError::*;

        let kind = match e {
            // User errors.
            GuestMacAddressInUse(_)
            | HostDeviceNameInUse(_)
            | DeviceIdNotFound
            | UpdateNotAllowedPostBoot => ErrorKind::User,
            // Internal errors.
            EpollHandlerNotFound(_) | RateLimiterUpdateFailed(_) => ErrorKind::Internal,
            OpenTap(ref te) => match te {
                // User errors.
                OpenTun(_) | CreateTap(_) | InvalidIfname => ErrorKind::User,
                // Internal errors.
                IoctlError(_) | CreateSocket(_) => ErrorKind::Internal,
            },
        };

        VmmActionError::NetworkConfig(kind, e)
    }
}

// It's convenient to turn StartMicrovmErrors into VmmActionErrors directly.
impl std::convert::From<StartMicrovmError> for VmmActionError {
    fn from(e: StartMicrovmError) -> Self {
        use StartMicrovmError::*;

        let kind = match e {
            // User errors.
            CreateVsockBackend(_)
            | CreateBlockDevice(_)
            | CreateNetDevice(_)
            | KernelCmdline(_)
            | KernelLoader(_)
            | MicroVMAlreadyRunning
            | MissingKernelConfig
            | NetDeviceNotConfigured
            | OpenBlockDevice(_)
            | VcpusNotConfigured => ErrorKind::User,
            // Internal errors.
            ConfigureSystem(_)
            | ConfigureVm(_)
            | CreateRateLimiter(_)
            | CreateVsockDevice(_)
            | DeviceManager
            | EventFd
            | GuestMemory(_)
            | RegisterBlockDevice(_)
            | RegisterEvent
            | RegisterMMIODevice(_)
            | RegisterNetDevice(_)
            | RegisterVsockDevice(_)
            | SeccompFilters(_)
            | Vcpu(_)
            | VcpuConfigure(_)
            | VcpuSpawn(_) => ErrorKind::Internal,
            #[cfg(target_arch = "x86_64")]
            LegacyIOBus(_) => ErrorKind::Internal,
            // The only user `LoadCommandline` error is `CommandLineOverflow`.
            LoadCommandline(ref cle) => match cle {
                kernel::cmdline::Error::CommandLineOverflow => ErrorKind::User,
                _ => ErrorKind::Internal,
            },
            StdinHandle(_) => ErrorKind::Internal,
        };
        VmmActionError::StartMicrovm(kind, e)
    }
}

impl VmmActionError {
    /// Returns the error type.
    pub fn kind(&self) -> &ErrorKind {
        use self::VmmActionError::*;

        match *self {
            BootSource(ref kind, _) => kind,
            DriveConfig(ref kind, _) => kind,
            Logger(ref kind, _) => kind,
            MachineConfig(ref kind, _) => kind,
            NetworkConfig(ref kind, _) => kind,
            StartMicrovm(ref kind, _) => kind,
            SendCtrlAltDel(ref kind, _) => kind,
            VsockConfig(ref kind, _) => kind,
        }
    }
}

impl Display for VmmActionError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::VmmActionError::*;
        match *self {
            BootSource(_, ref err) => write!(f, "{}", err),
            DriveConfig(_, ref err) => write!(f, "{}", err),
            Logger(_, ref err) => write!(f, "{}", err),
            MachineConfig(_, ref err) => write!(f, "{}", err),
            NetworkConfig(_, ref err) => write!(f, "{}", err),
            StartMicrovm(_, ref err) => write!(f, "{}", err),
            SendCtrlAltDel(_, ref err) => write!(f, "{}", err),
            VsockConfig(_, ref err) => write!(f, "{}", err),
        }
    }
}

/// Shorthand result type for external VMM commands.
pub type UserResult = std::result::Result<(), VmmActionError>;

/// Shorthand result type for internal VMM commands.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    use kernel;
    use net_util;

    // Helper function to get ErrorKind of error.
    fn error_kind<T: std::convert::Into<VmmActionError>>(err: T) -> ErrorKind {
        let err: VmmActionError = err.into();
        err.kind().clone()
    }

    #[test]
    fn test_drive_error_conversion() {
        // Test `DriveError` conversion
        assert_eq!(
            error_kind(DriveError::CannotOpenBlockDevice),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::InvalidBlockDevicePath),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::BlockDevicePathAlreadyExists),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::BlockDeviceUpdateFailed),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::OperationNotAllowedPreBoot),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::UpdateNotAllowedPostBoot),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(DriveError::RootBlockDeviceAlreadyAdded),
            ErrorKind::User
        );
    }

    #[test]
    fn test_vmconfig_error_conversion() {
        // Test `VmConfigError` conversion
        assert_eq!(error_kind(VmConfigError::InvalidVcpuCount), ErrorKind::User);
        assert_eq!(
            error_kind(VmConfigError::InvalidMemorySize),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(VmConfigError::UpdateNotAllowedPostBoot),
            ErrorKind::User
        );
    }

    #[test]
    fn test_network_interface_error_conversion() {
        // Test `NetworkInterfaceError` conversion
        assert_eq!(
            error_kind(NetworkInterfaceError::GuestMacAddressInUse(String::new())),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(NetworkInterfaceError::EpollHandlerNotFound(
                Error::DeviceEventHandlerNotFound
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(NetworkInterfaceError::HostDeviceNameInUse(String::new())),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(NetworkInterfaceError::DeviceIdNotFound),
            ErrorKind::User
        );
        // NetworkInterfaceError::OpenTap can be of multiple kinds.
        {
            assert_eq!(
                error_kind(NetworkInterfaceError::OpenTap(net_util::TapError::OpenTun(
                    io::Error::from_raw_os_error(0)
                ))),
                ErrorKind::User
            );
            assert_eq!(
                error_kind(NetworkInterfaceError::OpenTap(
                    net_util::TapError::CreateTap(io::Error::from_raw_os_error(0))
                )),
                ErrorKind::User
            );
            assert_eq!(
                error_kind(NetworkInterfaceError::OpenTap(
                    net_util::TapError::IoctlError(io::Error::from_raw_os_error(0))
                )),
                ErrorKind::Internal
            );
            assert_eq!(
                error_kind(NetworkInterfaceError::OpenTap(
                    net_util::TapError::CreateSocket(io::Error::from_raw_os_error(0))
                )),
                ErrorKind::Internal
            );
            assert_eq!(
                error_kind(NetworkInterfaceError::OpenTap(
                    net_util::TapError::InvalidIfname
                )),
                ErrorKind::User
            );
        }
        assert_eq!(
            error_kind(NetworkInterfaceError::RateLimiterUpdateFailed(
                devices::Error::FailedReadTap
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(NetworkInterfaceError::UpdateNotAllowedPostBoot),
            ErrorKind::User
        );
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_start_microvm_error_conversion_cl() {
        // Test `StartMicrovmError` conversion.
        #[cfg(target_arch = "x86_64")]
        assert_eq!(
            error_kind(StartMicrovmError::ConfigureSystem(
                arch::Error::ZeroPageSetup
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::ConfigureVm(
                vstate::Error::NotEnoughMemorySlots
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::CreateBlockDevice(
                io::Error::from_raw_os_error(0)
            )),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::CreateNetDevice(
                devices::virtio::Error::TapOpen(net_util::TapError::CreateTap(
                    io::Error::from_raw_os_error(0)
                ))
            )),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::CreateRateLimiter(
                io::Error::from_raw_os_error(0)
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::CreateVsockBackend(
                devices::virtio::vsock::VsockUnixBackendError::InvalidPortRequest
            )),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::CreateVsockDevice(
                devices::virtio::vsock::VsockError::NoData
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::DeviceManager),
            ErrorKind::Internal
        );
        assert_eq!(error_kind(StartMicrovmError::EventFd), ErrorKind::Internal);
        assert_eq!(
            error_kind(StartMicrovmError::GuestMemory(
                memory_model::GuestMemoryError::NoMemoryRegions
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::KernelCmdline(String::new())),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::KernelLoader(
                kernel::loader::Error::SeekKernelImage
            )),
            ErrorKind::User
        );
        #[cfg(target_arch = "x86_64")]
        assert_eq!(
            error_kind(StartMicrovmError::LegacyIOBus(
                device_manager::legacy::Error::EventFd(io::Error::from_raw_os_error(0))
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::LoadCommandline(
                kernel::cmdline::Error::CommandLineOverflow
            )),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::LoadCommandline(
                kernel::cmdline::Error::CommandLineCopy
            )),
            ErrorKind::Internal
        );
    }

    #[test]
    fn test_start_microvm_error_conversion_mv() {
        assert_eq!(
            error_kind(StartMicrovmError::MicroVMAlreadyRunning),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::MissingKernelConfig),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::NetDeviceNotConfigured),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::OpenBlockDevice(
                io::Error::from_raw_os_error(0)
            )),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::RegisterBlockDevice(
                device_manager::mmio::Error::IrqsExhausted
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::RegisterEvent),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::RegisterNetDevice(
                device_manager::mmio::Error::IrqsExhausted
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::RegisterMMIODevice(
                device_manager::mmio::Error::IrqsExhausted
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::RegisterVsockDevice(
                device_manager::mmio::Error::IrqsExhausted
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::SeccompFilters(
                seccomp::Error::InvalidArgumentNumber
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::Vcpu(vstate::Error::VcpuUnhandledKvmExit)),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::VcpuConfigure(
                vstate::Error::SetSupportedCpusFailed(io::Error::from_raw_os_error(0))
            )),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::VcpusNotConfigured),
            ErrorKind::User
        );
        assert_eq!(
            error_kind(StartMicrovmError::VcpuSpawn(io::Error::from_raw_os_error(
                0
            ))),
            ErrorKind::Internal
        );
        assert_eq!(
            error_kind(StartMicrovmError::StdinHandle(
                io::Error::from_raw_os_error(0)
            )),
            ErrorKind::Internal
        );
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_error_messages() {
        // Enum `Error`

        #[cfg(target_arch = "x86_64")]
        assert_eq!(
            format!(
                "{}",
                Error::CreateLegacyDevice(device_manager::legacy::Error::EventFd(
                    io::Error::from_raw_os_error(42)
                ))
            ),
            format!(
                "Error creating legacy device: {}",
                device_manager::legacy::Error::EventFd(io::Error::from_raw_os_error(42))
            )
        );
        assert_eq!(
            format!("{}", Error::EpollFd(io::Error::from_raw_os_error(42))),
            format!("Epoll fd error: {}", io::Error::from_raw_os_error(42))
        );
        assert_eq!(
            format!("{}", Error::EventFd(io::Error::from_raw_os_error(42))),
            format!("Event fd error: {}", io::Error::from_raw_os_error(42))
        );
        assert_eq!(
            format!("{}", Error::DeviceEventHandlerNotFound),
            "Device event handler not found. This might point to a guest device driver issue."
        );
        assert_eq!(
            format!("{}", Error::DeviceEventHandlerInvalidDowncast),
            "Device event handler couldn't be downcasted to expected type."
        );
        assert_eq!(
            format!("{:?}", Error::KvmContext(vstate::Error::KvmApiVersion(1))),
            format!("Failed to validate KVM support: {}", vstate::Error::KvmApiVersion(1))
        );
        assert_eq!(
            format!("{}", Error::Poll(io::Error::from_raw_os_error(42))),
            format!("Epoll wait failed: {}", io::Error::from_raw_os_error(42))
        );
        assert_eq!(
            format!("{}", Error::Serial(io::Error::from_raw_os_error(42))),
            format!(
                "Error writing to the serial console: {}",
                io::Error::from_raw_os_error(42)
            )
        );
        assert_eq!(
            format!("{}", Error::DeviceEventHandlerInvalidDowncast),
            "Device event handler couldn't be downcasted to expected type."
        );
        assert_eq!(
            format!("{}", Error::TimerFd(io::Error::from_raw_os_error(42))),
            format!(
                "Error creating timer fd: {}",
                io::Error::from_raw_os_error(42)
            )
        );
        assert_eq!(
            format!("{}", Error::Vm(vstate::Error::HTNotInitialized)),
            format!("Error opening VM fd: {}", vstate::Error::HTNotInitialized)
        );

        // Enum `ErrorKind`

        assert_ne!(ErrorKind::User, ErrorKind::Internal);
        assert_eq!(format!("{:?}", ErrorKind::User), "User");
        assert_eq!(format!("{:?}", ErrorKind::Internal), "Internal");

        // Enum VmmActionError

        assert_eq!(
            format!(
                "{}",
                VmmActionError::BootSource(
                    ErrorKind::User,
                    BootSourceConfigError::InvalidKernelCommandLine(
                        kernel::cmdline::Error::HasSpace.to_string()
                    )
                )
            ),
            format!(
                "{}",
                BootSourceConfigError::InvalidKernelCommandLine(
                    kernel::cmdline::Error::HasSpace.to_string()
                )
            )
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::DriveConfig(
                    ErrorKind::User,
                    DriveError::BlockDevicePathAlreadyExists
                )
            ),
            format!("{}", DriveError::BlockDevicePathAlreadyExists)
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::Logger(
                    ErrorKind::User,
                    LoggerConfigError::FlushMetrics(String::from("Failed to flush metrics"))
                )
            ),
            "Failed to flush metrics"
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::Logger(
                    ErrorKind::User,
                    LoggerConfigError::FlushMetrics(String::from("foobar"))
                )
            ),
            "foobar"
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::Logger(
                    ErrorKind::User,
                    LoggerConfigError::InitializationFailure(String::from(
                        "Failed to initialize logger"
                    ))
                )
            ),
            "Failed to initialize logger"
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::Logger(
                    ErrorKind::User,
                    LoggerConfigError::InitializationFailure(String::from("foobar"))
                )
            ),
            "foobar"
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::MachineConfig(ErrorKind::User, VmConfigError::InvalidMemorySize)
            ),
            format!("{}", VmConfigError::InvalidMemorySize)
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::NetworkConfig(
                    ErrorKind::User,
                    NetworkInterfaceError::DeviceIdNotFound
                )
            ),
            format!("{}", NetworkInterfaceError::DeviceIdNotFound)
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::StartMicrovm(ErrorKind::User, StartMicrovmError::EventFd)
            ),
            format!("{}", StartMicrovmError::EventFd)
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::SendCtrlAltDel(
                    ErrorKind::User,
                    I8042DeviceError::InternalBufferFull
                )
            ),
            format!("{}", I8042DeviceError::InternalBufferFull)
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::SendCtrlAltDel(
                    ErrorKind::User,
                    I8042DeviceError::InternalBufferFull
                )
            ),
            I8042DeviceError::InternalBufferFull.to_string()
        );
        assert_eq!(
            VmmActionError::SendCtrlAltDel(ErrorKind::User, I8042DeviceError::InternalBufferFull)
                .kind(),
            &ErrorKind::User
        );
        assert_eq!(
            format!(
                "{}",
                VmmActionError::VsockConfig(ErrorKind::User, VsockError::UpdateNotAllowedPostBoot)
            ),
            format!("{}", VsockError::UpdateNotAllowedPostBoot)
        );

        assert_eq!(
            format!(
                "{}",
                VmmActionError::VsockConfig(ErrorKind::User, VsockError::UpdateNotAllowedPostBoot)
            ),
            "The update operation is not allowed after boot."
        );
    }
}
