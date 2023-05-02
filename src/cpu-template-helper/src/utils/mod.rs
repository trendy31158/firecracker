// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use vmm::builder::{build_microvm_for_boot, StartMicrovmError};
use vmm::resources::VmResources;
use vmm::seccomp_filters::{get_filters, SeccompConfig};
use vmm::vmm_config::instance_info::{InstanceInfo, VmState};
use vmm::{EventManager, Vmm, HTTP_MAX_PAYLOAD_SIZE};

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

const CPU_TEMPLATE_HELPER_VERSION: &str = env!("FIRECRACKER_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to create VmResources.
    #[error("Failed to create VmResources: {0}")]
    CreateVmResources(vmm::resources::Error),
    /// Failed to build microVM.
    #[error("Failed to build microVM: {0}")]
    BuildMicroVm(#[from] StartMicrovmError),
}

#[allow(dead_code)]
pub fn build_microvm_from_config(config: &str) -> Result<(Arc<Mutex<Vmm>>, VmResources), Error> {
    // Prepare resources from the given config file.
    let instance_info = InstanceInfo {
        id: "anonymous-instance".to_string(),
        state: VmState::NotStarted,
        vmm_version: CPU_TEMPLATE_HELPER_VERSION.to_string(),
        app_name: "cpu-template-helper".to_string(),
    };
    let vm_resources = VmResources::from_json(config, &instance_info, HTTP_MAX_PAYLOAD_SIZE, None)
        .map_err(Error::CreateVmResources)?;
    let mut event_manager = EventManager::new().unwrap();
    let seccomp_filters = get_filters(SeccompConfig::None).unwrap();

    // Build a microVM.
    let vmm = build_microvm_for_boot(
        &instance_info,
        &vm_resources,
        &mut event_manager,
        &seccomp_filters,
    )?;

    Ok((vmm, vm_resources))
}

pub fn add_suffix(path: &Path, suffix: &str) -> PathBuf {
    // Extract the part of the filename before the extension.
    let mut new_file_name = OsString::from(path.file_stem().unwrap());

    // Push the suffix and the extension.
    new_file_name.push(suffix);
    if let Some(ext) = path.extension() {
        new_file_name.push(".");
        new_file_name.push(ext);
    }

    // Swap the file name.
    path.with_file_name(new_file_name)
}

#[cfg(test)]
mod tests {
    use utils::tempfile::TempFile;
    use vmm::utilities::mock_resources::kernel_image_path;

    use super::*;
    use crate::tests::generate_config;

    const SUFFIX: &str = "_suffix";

    #[test]
    fn test_build_microvm_from_valid_config() {
        let kernel_image_path = kernel_image_path(None);
        let rootfs_file = TempFile::new().unwrap();
        let valid_config =
            generate_config(&kernel_image_path, rootfs_file.as_path().to_str().unwrap());

        build_microvm_from_config(&valid_config).unwrap();
    }

    #[test]
    fn test_build_microvm_from_invalid_config() {
        let rootfs_file = TempFile::new().unwrap();
        let invalid_config = generate_config(
            "/invalid_kernel_image_path",
            rootfs_file.as_path().to_str().unwrap(),
        );

        match build_microvm_from_config(&invalid_config) {
            Ok(_) => panic!("Should fail with `No such file or directory`."),
            Err(Error::CreateVmResources(_)) => (),
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }

    #[test]
    fn test_add_suffix_filename_only() {
        let path = PathBuf::from("file.ext");
        let expected = PathBuf::from(format!("file{SUFFIX}.ext"));
        assert_eq!(add_suffix(&path, SUFFIX), expected);
    }

    #[test]
    fn test_add_suffix_filename_without_ext() {
        let path = PathBuf::from("file_no_ext");
        let expected = PathBuf::from(format!("file_no_ext{SUFFIX}"));
        assert_eq!(add_suffix(&path, SUFFIX), expected);
    }

    #[test]
    fn test_add_suffix_rel_path() {
        let path = PathBuf::from("relative/path/to/file.ext");
        let expected = PathBuf::from(format!("relative/path/to/file{SUFFIX}.ext"));
        assert_eq!(add_suffix(&path, SUFFIX), expected);
    }

    #[test]
    fn test_add_suffix_abs_path() {
        let path = PathBuf::from("/absolute/path/to/file.ext");
        let expected = PathBuf::from(format!("/absolute/path/to/file{SUFFIX}.ext"));
        assert_eq!(add_suffix(&path, SUFFIX), expected);
    }
}
