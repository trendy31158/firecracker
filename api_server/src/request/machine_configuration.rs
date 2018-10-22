use std::result;

use futures::sync::oneshot;
use hyper::{Method, Response, StatusCode};

use data_model::vm::VmConfig;
use http_service::json_response;
use request::{GenerateHyperResponse, IntoParsedRequest, ParsedRequest};
use vmm::VmmAction;

impl GenerateHyperResponse for VmConfig {
    fn generate_response(&self) -> Response {
        let vcpu_count = self.vcpu_count.unwrap_or(1);
        let mem_size = self.mem_size_mib.unwrap_or(128);
        let ht_enabled = self.ht_enabled.unwrap_or(false);
        let cpu_template = self
            .cpu_template
            .map_or("Uninitialized".to_string(), |c| c.to_string());

        json_response(
            StatusCode::Ok,
            format!(
                "{{ \"vcpu_count\": {:?}, \"mem_size_mib\": {:?},  \"ht_enabled\": {:?},  \"cpu_template\": {:?} }}",
                vcpu_count, mem_size, ht_enabled, cpu_template
            ),
        )
    }
}

impl IntoParsedRequest for VmConfig {
    fn into_parsed_request(
        self,
        _: Option<String>,
        method: Method,
    ) -> result::Result<ParsedRequest, String> {
        let (sender, receiver) = oneshot::channel();
        match method {
            Method::Get => Ok(ParsedRequest::Sync(
                VmmAction::GetVmConfiguration(sender),
                receiver,
            )),
            Method::Put => {
                if self.vcpu_count.is_none()
                    && self.mem_size_mib.is_none()
                    && self.cpu_template.is_none()
                    && self.ht_enabled.is_none()
                {
                    return Err(String::from("Empty request."));
                }
                Ok(ParsedRequest::Sync(
                    VmmAction::SetVmConfiguration(self, sender),
                    receiver,
                ))
            }
            _ => Err(String::from("Invalid method.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_model::vm::CpuFeaturesTemplate;

    #[test]
    fn test_into_parsed_request() {
        let body = VmConfig {
            vcpu_count: Some(8),
            mem_size_mib: Some(1024),
            ht_enabled: Some(true),
            cpu_template: Some(CpuFeaturesTemplate::T2),
        };
        let (sender, receiver) = oneshot::channel();
        assert!(
            body.clone()
                .into_parsed_request(None, Method::Put)
                .eq(&Ok(ParsedRequest::Sync(
                    VmmAction::SetVmConfiguration(body, sender),
                    receiver
                )))
        );
        let uninitialized = VmConfig {
            vcpu_count: None,
            mem_size_mib: None,
            ht_enabled: None,
            cpu_template: None,
        };
        assert!(
            uninitialized
                .clone()
                .into_parsed_request(None, Method::Get)
                .is_ok()
        );
        assert!(
            uninitialized
                .clone()
                .into_parsed_request(None, Method::Patch)
                .is_err()
        );

        match uninitialized.into_parsed_request(None, Method::Put) {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, String::from("Empty request.")),
        };
    }
}
