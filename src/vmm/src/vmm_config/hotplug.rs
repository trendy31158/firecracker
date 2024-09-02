// Copyright 2024 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io;

use serde::{Deserialize, Serialize};

use crate::vstate::vcpu::{KvmVcpuConfigureError, VcpuError};
use crate::{StartVcpusError, VmmError};
/// Unifying enum for all types of hotplug request configs.
/// Currently only Vcpus may be hotplugged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotplugRequestConfig {
    /// Vcpu hotplug request
    Vcpu(HotplugVcpuConfig),
}

/// Errors related to different types of hotplugging.
/// Currently only Vcpus can be hotplugged.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum HotplugRequestError {
    /// Vcpu hotplugging error: {0}
    Vcpu(#[from] HotplugVcpuError),
}

/// Errors associated with hot-plugging vCPUs
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum HotplugVcpuError {
    /// The number of vCPUs added must be greater than 0.
    VcpuCountTooLow,
    /// The number of vCPUs added must be less than 32.
    VcpuCountTooHigh,
    /// Event fd error: {0}
    EventFd(#[from] io::Error),
    /// Error creating the vcpu: {0}
    VcpuCreate(VcpuError),
    /// Error configuring KVM vcpu: {0}
    VcpuConfigure(#[from] KvmVcpuConfigureError),
    /// Failed to start vCPUs
    VcpuStart(StartVcpusError),
    /// No seccomp filter for thread category: {0}
    MissingSeccompFilters(String),
    /// Error resuming VM: {0}
    VmResume(#[from] VmmError),
    /// Cannot hotplug vCPUs after restoring from snapshot
    RestoredFromSnapshot,
}

/// Config for hotplugging vCPUS
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HotplugVcpuConfig {
    /// Number of vcpus after operation.
    pub target: u8,
}
