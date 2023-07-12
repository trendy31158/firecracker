// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]

//! Crate that implements Firecracker specific functionality as far as logging and metrics
//! collecting.

mod metrics;

use std::sync::OnceLock;

use tracing::warn;

#[cfg(target_arch = "aarch64")]
pub use crate::metrics::RTCDeviceMetrics;
pub use crate::metrics::{
    IncMetric, MetricsError, ProcessTimeReporter, SerialDeviceMetrics, SharedIncMetric,
    SharedStoreMetric, StoreMetric, METRICS,
};

/// Alias for `std::io::LineWriter<std::fs::File>`.
pub type FcLineWriter = std::io::LineWriter<std::fs::File>;

/// Prefix to be used in log lines for functions/modules in Firecracker
/// that are not generally available.
const DEV_PREVIEW_LOG_PREFIX: &str = "[DevPreview]";

/// The default instance ID.
pub const DEFAULT_INSTANCE_ID: &str = "anonymous-instance";
/// The instance ID to use when initializing the logger.
pub static INSTANCE_ID: OnceLock<String> = OnceLock::new();

/// Log a standard warning message indicating a given feature name
/// is in development preview.
#[tracing::instrument(level = "debug", ret(skip), skip(feature_name, msg_opt))]
pub fn log_dev_preview_warning(feature_name: &str, msg_opt: Option<String>) {
    match msg_opt {
        None => warn!("{DEV_PREVIEW_LOG_PREFIX} {feature_name} is in development preview."),
        Some(msg) => {
            warn!("{DEV_PREVIEW_LOG_PREFIX} {feature_name} is in development preview - {msg}")
        }
    }
}

/// Helper function for updating the value of a store metric with elapsed time since some time in a
/// past.
#[tracing::instrument(level = "debug", ret(skip), skip(metric, start_time_us))]
pub fn update_metric_with_elapsed_time(metric: &SharedStoreMetric, start_time_us: u64) -> u64 {
    let delta_us = utils::time::get_time_us(utils::time::ClockType::Monotonic) - start_time_us;
    metric.store(delta_us as usize);
    delta_us
}
