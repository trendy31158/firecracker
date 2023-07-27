// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Implements a virtio network device.

use std::io;

/// Maximum size of the frame buffers handled by this device.
pub const MAX_BUFFER_SIZE: usize = 65562;
/// Queue size for network device.
pub const NET_QUEUE_SIZE: u16 = 256;
/// The number of queues of the network device.
pub const NET_NUM_QUEUES: usize = 2;
pub const NET_QUEUE_SIZES: [u16; NET_NUM_QUEUES] = [NET_QUEUE_SIZE; NET_NUM_QUEUES];
/// The index of the rx queue from Net device queues/queues_evts vector.
pub const RX_INDEX: usize = 0;
/// The index of the tx queue from Net device queues/queues_evts vector.
pub const TX_INDEX: usize = 1;

pub mod device;
mod event_handler;
pub mod persist;
mod tap;
pub mod test_utils;

pub use tap::{Tap, TapError};

pub use self::device::Net;
pub use self::event_handler::*;

/// Enum representing the Net device queue types
#[derive(Debug)]
pub enum NetQueue {
    /// The RX queue
    Rx,
    /// The TX queue
    Tx,
}

/// Errors the network device can trigger.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    /// Open tap device failed
    #[error("Open tap device failed: {0}")]
    TapOpen(TapError),
    /// Setting tap interface offload flags failed
    #[error("Setting tap interface offload flags failed: {0}")]
    TapSetOffload(TapError),
    /// Setting vnet header size failed
    #[error("Setting vnet header size failed: {0}")]
    TapSetVnetHdrSize(TapError),
    /// EventFd error
    #[error("EventFd error: {0}")]
    EventFd(io::Error),
    /// IO error
    #[error("IO error: {0}")]
    IO(io::Error),
    /// The VNET header is missing from the frame
    #[error("The VNET header is missing from the frame")]
    VnetHeaderMissing,
}
