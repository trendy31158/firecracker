// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use super::super::VmmAction;
#[cfg(target_arch = "x86_64")]
use request::StatusCode;
use request::{Body, Error, ParsedRequest};
#[cfg(target_arch = "x86_64")]
use vmm::vmm_config::snapshot::{CreateSnapshotParams, LoadSnapshotParams};
use vmm::vmm_config::snapshot::{Vm, VmState};
#[cfg(target_arch = "x86_64")]
use Method;

#[cfg(target_arch = "x86_64")]
pub fn parse_put_snapshot(
    body: &Body,
    request_type_from_path: Option<&&str>,
) -> Result<ParsedRequest, Error> {
    match request_type_from_path {
        Some(&request_type) => match request_type {
            "create" => Ok(ParsedRequest::Sync(VmmAction::CreateSnapshot(
                serde_json::from_slice::<CreateSnapshotParams>(body.raw())
                    .map_err(Error::SerdeJson)?,
            ))),
            "load" => Ok(ParsedRequest::Sync(VmmAction::LoadSnapshot(
                serde_json::from_slice::<LoadSnapshotParams>(body.raw())
                    .map_err(Error::SerdeJson)?,
            ))),
            _ => Err(Error::InvalidPathMethod(
                format!("/snapshot/{}", request_type),
                Method::Put,
            )),
        },
        None => Err(Error::Generic(
            StatusCode::BadRequest,
            "Missing snapshot operation type.".to_string(),
        )),
    }
}

pub fn parse_patch_vm_state(body: &Body) -> Result<ParsedRequest, Error> {
    let vm = serde_json::from_slice::<Vm>(body.raw()).map_err(Error::SerdeJson)?;

    match vm.state {
        VmState::Paused => Ok(ParsedRequest::Sync(VmmAction::Pause)),
        VmState::Resumed => Ok(ParsedRequest::Sync(VmmAction::Resume)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_parse_put_snapshot() {
        use std::path::PathBuf;
        use vmm::vmm_config::snapshot::SnapshotType;

        let mut body = r#"{
                "snapshot_type": "Diff",
                "snapshot_path": "foo",
                "mem_file_path": "bar",
                "version": 2
              }"#;

        let mut expected_cfg = CreateSnapshotParams {
            snapshot_type: SnapshotType::Diff,
            snapshot_path: PathBuf::from("foo"),
            mem_file_path: PathBuf::from("bar"),
            version: Some(2),
        };

        match parse_put_snapshot(&Body::new(body), Some(&"create")) {
            Ok(ParsedRequest::Sync(VmmAction::CreateSnapshot(cfg))) => {
                assert_eq!(cfg, expected_cfg)
            }
            _ => panic!("Test failed."),
        }

        body = r#"{
                "snapshot_path": "foo",
                "mem_file_path": "bar"
              }"#;

        expected_cfg = CreateSnapshotParams {
            snapshot_type: SnapshotType::Full,
            snapshot_path: PathBuf::from("foo"),
            mem_file_path: PathBuf::from("bar"),
            version: None,
        };

        match parse_put_snapshot(&Body::new(body), Some(&"create")) {
            Ok(ParsedRequest::Sync(VmmAction::CreateSnapshot(cfg))) => {
                assert_eq!(cfg, expected_cfg)
            }
            _ => panic!("Test failed."),
        }

        let invalid_body = r#"{
                "invalid_field": "foo",
                "mem_file_path": "bar"
              }"#;

        assert!(parse_put_snapshot(&Body::new(invalid_body), Some(&"create")).is_err());

        body = r#"{
                "snapshot_path": "foo",
                "mem_file_path": "bar"
              }"#;

        let mut expected_cfg = LoadSnapshotParams {
            snapshot_path: PathBuf::from("foo"),
            mem_file_path: PathBuf::from("bar"),
            enable_diff_snapshots: false,
        };
        match parse_put_snapshot(&Body::new(body), Some(&"load")) {
            Ok(ParsedRequest::Sync(VmmAction::LoadSnapshot(cfg))) => assert_eq!(cfg, expected_cfg),
            _ => panic!("Test failed."),
        }

        body = r#"{
                "snapshot_path": "foo",
                "mem_file_path": "bar",
                "enable_diff_snapshots": true
              }"#;

        expected_cfg = LoadSnapshotParams {
            snapshot_path: PathBuf::from("foo"),
            mem_file_path: PathBuf::from("bar"),
            enable_diff_snapshots: true,
        };

        match parse_put_snapshot(&Body::new(body), Some(&"load")) {
            Ok(ParsedRequest::Sync(VmmAction::LoadSnapshot(cfg))) => assert_eq!(cfg, expected_cfg),
            _ => panic!("Test failed."),
        }

        assert!(parse_put_snapshot(&Body::new(body), Some(&"invalid")).is_err());
        assert!(parse_put_snapshot(&Body::new(body), None).is_err());
    }

    #[test]
    fn test_parse_patch_vm_state() {
        let mut body = r#"{
                "state": "Paused"
              }"#;

        assert!(parse_patch_vm_state(&Body::new(body))
            .unwrap()
            .eq(&ParsedRequest::Sync(VmmAction::Pause)));

        body = r#"{
                "state": "Resumed"
              }"#;

        assert!(parse_patch_vm_state(&Body::new(body))
            .unwrap()
            .eq(&ParsedRequest::Sync(VmmAction::Resume)));

        let invalid_body = r#"{
                "invalid": "Paused"
              }"#;

        assert!(parse_patch_vm_state(&Body::new(invalid_body)).is_err());
    }
}
