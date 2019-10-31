// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io;

/// Default guest kernel command line:
/// - `reboot=k` shut down the guest on reboot, instead of well... rebooting;
/// - `panic=1` on panic, reboot after 1 second;
/// - `pci=off` do not scan for PCI devices (save boot time);
/// - `nomodules` disable loadable kernel module support;
/// - `8250.nr_uarts=0` disable 8250 serial interface;
/// - `i8042.noaux` do not probe the i8042 controller for an attached mouse (save boot time);
/// - `i8042.nomux` do not probe i8042 for a multiplexing controller (save boot time);
/// - `i8042.nopnp` do not use ACPIPnP to discover KBD/AUX controllers (save boot time);
/// - `i8042.dumbkbd` do not attempt to control kbd state via the i8042 (save boot time).
pub const DEFAULT_KERNEL_CMDLINE: &str = "reboot=k panic=1 pci=off nomodules 8250.nr_uarts=0 \
                                          i8042.noaux i8042.nomux i8042.nopnp i8042.dumbkbd";

/// Holds the kernel configuration.
pub struct KernelConfig {
    /// The commandline validated against correctness.
    pub cmdline: kernel::cmdline::Cmdline,
    /// The descriptor to the kernel file.
    pub kernel_file: File,
}

/// Strongly typed data structure used to configure the boot source of the
/// microvm.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BootSourceConfig {
    /// Path of the kernel image.
    pub kernel_image_path: String,
    /// The boot arguments to pass to the kernel. If this field is uninitialized, the default
    /// kernel command line is used: `reboot=k panic=1 pci=off nomodules 8250.nr_uarts=0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_args: Option<String>,
}

/// Errors associated with actions on `BootSourceConfig`.
#[derive(Debug)]
pub enum BootSourceConfigError {
    /// The kernel file cannot be opened.
    InvalidKernelPath(io::Error),
    /// The kernel command line is invalid.
    InvalidKernelCommandLine(String),
    /// The boot source cannot be update post boot.
    UpdateNotAllowedPostBoot,
}

impl Display for BootSourceConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::BootSourceConfigError::*;
        match *self {
            InvalidKernelPath(ref e) => write!(f, "The kernel file cannot be opened: {}", e),
            InvalidKernelCommandLine(ref e) => {
                write!(f, "The kernel command line is invalid: {}", e.as_str())
            }
            UpdateNotAllowedPostBoot => {
                write!(f, "The update operation is not allowed after boot.")
            }
        }
    }
}
