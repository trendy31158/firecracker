// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Implements a virtio balloon device.

mod device;
mod event_handler;
pub mod persist;
pub mod test_utils;
mod utils;

pub use crate::virtio::balloon::device::{Balloon, BalloonConfig, BalloonStats};
use crate::virtio::queue::QueueError;

/// Device ID used in MMIO device identification.
/// Because Balloon is unique per-vm, this ID can be hardcoded.
pub const BALLOON_DEV_ID: &str = "balloon";
/// The size of the config space.
pub const CONFIG_SPACE_SIZE: usize = 8;
/// Max size of virtio queues.
pub const QUEUE_SIZE: u16 = 256;
/// Number of virtio queues.
pub const NUM_QUEUES: usize = 3;
/// Virtio queue sizes, in number of descriptor chain heads.
//  There are 3 queues for a virtio device (in this order): RX, TX, Event
pub const QUEUE_SIZES: &[u16] = &[QUEUE_SIZE, QUEUE_SIZE, QUEUE_SIZE];
/// Number of 4K pages in a MiB.
pub const MIB_TO_4K_PAGES: u32 = 256;
/// The maximum number of pages that can be received in a single descriptor.
pub const MAX_PAGES_IN_DESC: usize = 256;
/// The maximum number of pages that can be compacted into ranges during process_inflate().
/// Needs to be a multiple of MAX_PAGES_IN_DESC.
pub const MAX_PAGE_COMPACT_BUFFER: usize = 2048;
/// The addresses given by the driver are divided by 4096.
pub const VIRTIO_BALLOON_PFN_SHIFT: u32 = 12;
/// The index of the deflate queue from Balloon device queues/queues_evts vector.
pub const INFLATE_INDEX: usize = 0;
/// The index of the deflate queue from Balloon device queues/queues_evts vector.
pub const DEFLATE_INDEX: usize = 1;
/// The index of the deflate queue from Balloon device queues/queues_evts vector.
pub const STATS_INDEX: usize = 2;

// The feature bitmap for virtio balloon.
const VIRTIO_BALLOON_F_STATS_VQ: u32 = 1; // Enable statistics.
const VIRTIO_BALLOON_F_DEFLATE_ON_OOM: u32 = 2; // Deflate balloon on OOM.

// The statistics tags.
const VIRTIO_BALLOON_S_SWAP_IN: u16 = 0;
const VIRTIO_BALLOON_S_SWAP_OUT: u16 = 1;
const VIRTIO_BALLOON_S_MAJFLT: u16 = 2;
const VIRTIO_BALLOON_S_MINFLT: u16 = 3;
const VIRTIO_BALLOON_S_MEMFREE: u16 = 4;
const VIRTIO_BALLOON_S_MEMTOT: u16 = 5;
const VIRTIO_BALLOON_S_AVAIL: u16 = 6;
const VIRTIO_BALLOON_S_CACHES: u16 = 7;
const VIRTIO_BALLOON_S_HTLB_PGALLOC: u16 = 8;
const VIRTIO_BALLOON_S_HTLB_PGFAIL: u16 = 9;

/// Balloon device related errors.
#[derive(Debug)]
pub enum Error {
    /// No balloon device found.
    DeviceNotFound,
    /// Device not activated yet.
    DeviceNotActive,
    /// EventFd error.
    EventFd(std::io::Error),
    /// Received error while sending an interrupt.
    InterruptError(std::io::Error),
    /// Guest gave us a malformed descriptor.
    MalformedDescriptor,
    /// Guest gave us a malformed payload.
    MalformedPayload,
    /// Error restoring the balloon device queues.
    QueueRestoreError,
    /// Received stats querry when stats are disabled.
    StatisticsDisabled,
    /// Statistics cannot be enabled/disabled after activation.
    StatisticsStateChange,
    /// Amount of pages requested cannot fit in `u32`.
    TooManyPagesRequested,
    /// Error while processing the virt queues.
    Queue(QueueError),
    /// Error creating the statistics timer.
    Timer(std::io::Error),
}
