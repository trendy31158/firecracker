// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

pub mod device;
pub mod event_handler;

pub use self::device::{Entropy, Error};

pub(crate) const NUM_QUEUES: usize = 1;
pub(crate) const QUEUE_SIZE: u16 = 256;

pub(crate) const RNG_QUEUE: usize = 0;
