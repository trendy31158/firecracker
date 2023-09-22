// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Implements virtio devices, queues, and transport mechanisms.

use std::any::Any;
use std::io::Error as IOError;

pub mod balloon;
pub mod block;
pub mod device;
mod iovec;
mod mmio;
pub mod net;
pub mod persist;
mod queue;
pub mod rng;
pub mod test_utils;
pub mod vsock;

pub use self::balloon::*;
pub use self::block::*;
pub use self::device::*;
pub use self::mmio::*;
pub use self::net::*;
pub use self::persist::*;
pub use self::queue::*;
pub use self::rng::*;
pub use self::vsock::*;
use crate::arch::DeviceSubtype;

/// When the driver initializes the device, it lets the device know about the
/// completed stages using the Device Status Field.
///
/// These following consts are defined in the order in which the bits would
/// typically be set by the driver. INIT -> ACKNOWLEDGE -> DRIVER and so on.
///
/// This module is a 1:1 mapping for the Device Status Field in the virtio 1.0
/// specification, section 2.1.
mod device_status {
    pub const INIT: u32 = 0;
    pub const ACKNOWLEDGE: u32 = 1;
    pub const DRIVER: u32 = 2;
    pub const FAILED: u32 = 128;
    pub const FEATURES_OK: u32 = 8;
    pub const DRIVER_OK: u32 = 4;
}

/// Types taken from linux/virtio_ids.h.
/// Type 0 is not used by virtio. Use it as wildcard for non-virtio devices
/// Virtio net device ID.
pub const TYPE_NET: u32 = 1;
/// Virtio block device ID.
pub const TYPE_BLOCK: u32 = 2;
/// Virtio rng device ID.
pub const TYPE_RNG: u32 = 4;
/// Virtio balloon device ID.
pub const TYPE_BALLOON: u32 = 5;

/// Type 0 is not used by virtio. Use it as wildcard for non-virtio devices
pub const SUBTYPE_NON_VIRTIO: DeviceSubtype = 0;
pub const SUBTYPE_NET: DeviceSubtype = 1;
pub const SUBTYPE_BLOCK: DeviceSubtype = 1;
pub const SUBTYPE_RNG: DeviceSubtype = 1;
pub const SUBTYPE_BALLOON: DeviceSubtype = 1;
pub const SUBTYPE_VSOCK: DeviceSubtype = 1;

/// Offset from the base MMIO address of a virtio device used by the guest to notify the device of
/// queue events.
pub const NOTIFY_REG_OFFSET: u32 = 0x50;

/// Errors triggered when activating a VirtioDevice.
#[derive(Debug, thiserror::Error)]
pub enum ActivateError {
    /// Epoll error.
    #[error("epollctl: {0}")]
    EpollCtl(IOError),
    /// General error at activation.
    #[error("General error at activation")]
    BadActivate,
}

/// Trait that helps in upcasting an object to Any
pub trait AsAny {
    /// Return the immutable any encapsulated object.
    fn as_any(&self) -> &dyn Any;

    /// Return the mutable encapsulated any object.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
