// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Serialize};

pub mod device;
pub mod persist;
pub mod vhost_user;
pub mod virtio;

pub use virtio::device::VirtioBlock;
pub use virtio::VirtioBlockError;

/// Configuration options for disk caching.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum CacheType {
    /// Flushing mechanic not will be advertised to the guest driver
    #[default]
    Unsafe,
    /// Flushing mechanic will be advertised to the guest driver and
    /// flush requests coming from the guest will be performed using
    /// `fsync`.
    Writeback,
}
