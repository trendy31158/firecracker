// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

pub mod device;
mod event_handler;
pub mod persist;

pub use self::device::BlockVhostUser;
use crate::devices::virtio::vhost_user::VhostUserError;

/// Number of queues for the vhost-user block device.
pub const NUM_QUEUES: u64 = 1;

/// Queue size for the vhost-user block device.
pub const QUEUE_SIZE: u16 = 256;

/// Vhost-user block device error.
#[derive(Debug, thiserror::Error)]
pub enum BlockVhostUserError {
    // Persistence error.
    #[error("Persistence error: {0}")]
    Persist(crate::devices::virtio::persist::PersistError),
    // Vhost-user error.
    #[error("Vhost-user error: {0}")]
    VhostUser(VhostUserError),
    // Vhost error.
    #[error("Vhost error: {0}")]
    Vhost(vhost::Error),
    // Error opening eventfd.
    #[error("Error opening eventfd: {0}")]
    EventFd(std::io::Error),
    // Error creating an irqfd.
    #[error("Error creating irqfd: {0}")]
    IrqTrigger(std::io::Error),
}
