// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
//! Implements the interface for intercepting API requests, forwarding them to the VMM
//! and responding to the user.
//! It is constructed on top of an HTTP Server that uses Unix Domain Sockets and `EPOLL` to
//! handle multiple connections on the same thread.
mod parsed_request;
mod request;

use serde_json::json;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::{fmt, io};

use crate::parsed_request::{ParsedRequest, RequestAction};
use logger::{
    debug, error, info, update_metric_with_elapsed_time, warn, ProcessTimeReporter, METRICS,
};
pub use micro_http::{
    Body, HttpServer, Method, Request, RequestError, Response, ServerError, ServerRequest,
    ServerResponse, StatusCode, Version,
};
use mmds::data_store;
use mmds::data_store::Mmds;
use seccompiler::BpfProgramRef;
use utils::eventfd::EventFd;
use vmm::rpc_interface::{VmmAction, VmmActionError, VmmData};
use vmm::vmm_config::snapshot::SnapshotType;

/// Shorthand type for a request containing a boxed VmmAction.
pub type ApiRequest = Box<VmmAction>;
/// Shorthand type for a response containing a boxed Result.
pub type ApiResponse = Box<std::result::Result<VmmData, VmmActionError>>;

/// Errors thrown when binding the API server to the socket path.
pub enum Error {
    /// IO related error.
    Io(io::Error),
    /// EventFD related error.
    Eventfd(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Eventfd(ref err) => write!(f, "EventFd error: {}", err),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Eventfd(ref err) => write!(f, "EventFd error: {}", err),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Structure associated with the API server implementation.
pub struct ApiServer {
    /// MMDS info directly accessible from the API thread.
    mmds_info: Arc<Mutex<Mmds>>,
    /// Sender which allows passing messages to the VMM.
    api_request_sender: mpsc::Sender<ApiRequest>,
    /// Receiver which collects messages from the VMM.
    vmm_response_receiver: mpsc::Receiver<ApiResponse>,
    /// FD on which we notify the VMM that we have sent at least one
    /// `VmmRequest`.
    to_vmm_fd: EventFd,
    /// If this flag is set, the API thread will go down.
    shutdown_flag: bool,
}

impl ApiServer {
    /// Constructor for `ApiServer`.
    ///
    /// Returns the newly formed `ApiServer`.
    pub fn new(
        mmds_info: Arc<Mutex<Mmds>>,
        api_request_sender: mpsc::Sender<ApiRequest>,
        vmm_response_receiver: mpsc::Receiver<ApiResponse>,
        to_vmm_fd: EventFd,
    ) -> Self {
        ApiServer {
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
            shutdown_flag: false,
        }
    }

    /// Starts the HTTP Server by binding to the socket path provided as
    /// an argument.
    ///
    /// # Arguments
    ///
    /// * `path` - the socket path on which the server will wait for requests.
    /// * `start_time_us` - the timestamp for when the process was started in us.
    /// * `start_time_cpu_us` - the timestamp for when the process was started in CPU us.
    /// * `seccomp_filter` - the seccomp filter to apply.
    ///
    /// # Example
    ///
    /// ```
    /// use api_server::ApiServer;
    /// use logger::ProcessTimeReporter;
    /// use mmds::MAX_DATA_STORE_SIZE;
    /// use mmds::MMDS;
    /// use std::env::consts::ARCH;
    /// use std::{
    ///     convert::TryInto, io::Read, io::Write, os::unix::net::UnixStream, path::PathBuf,
    ///     sync::mpsc::channel, thread, time::Duration,
    /// };
    /// use utils::{eventfd::EventFd, tempfile::TempFile};
    /// use vmm::rpc_interface::VmmData;
    /// use vmm::seccomp_filters::{get_filters, SeccompConfig};
    /// use vmm::vmm_config::instance_info::InstanceInfo;
    ///
    /// let mut tmp_socket = TempFile::new().unwrap();
    /// tmp_socket.remove().unwrap();
    /// let path_to_socket = tmp_socket.as_path().to_str().unwrap().to_owned();
    /// let api_thread_path_to_socket = path_to_socket.clone();
    /// let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
    /// let (api_request_sender, _from_api) = channel();
    /// let (to_api, vmm_response_receiver) = channel();
    /// let mmds_info = MMDS.clone();
    /// let time_reporter = ProcessTimeReporter::new(Some(1), Some(1), Some(1));
    /// let seccomp_filters = get_filters(SeccompConfig::None).unwrap();
    /// let payload_limit = Some(MAX_DATA_STORE_SIZE);
    ///
    /// thread::Builder::new()
    ///     .name("fc_api_test".to_owned())
    ///     .spawn(move || {
    ///         ApiServer::new(
    ///             mmds_info,
    ///             api_request_sender,
    ///             vmm_response_receiver,
    ///             to_vmm_fd,
    ///         )
    ///         .bind_and_run(
    ///             PathBuf::from(api_thread_path_to_socket),
    ///             time_reporter,
    ///             seccomp_filters.get("api").unwrap(),
    ///             payload_limit,
    ///         )
    ///         .unwrap();
    ///     })
    ///     .unwrap();
    ///
    /// thread::sleep(Duration::from_millis(10));
    /// to_api
    ///     .send(Box::new(Ok(VmmData::InstanceInformation(
    ///         InstanceInfo::default(),
    ///     ))))
    ///     .unwrap();
    /// let mut sock = UnixStream::connect(PathBuf::from(path_to_socket)).unwrap();
    /// // Send a GET instance-info request.
    /// assert!(sock.write_all(b"GET / HTTP/1.1\r\n\r\n").is_ok());
    /// let mut buf: [u8; 100] = [0; 100];
    /// assert!(sock.read(&mut buf[..]).unwrap() > 0);
    /// ```
    pub fn bind_and_run(
        &mut self,
        path: PathBuf,
        process_time_reporter: ProcessTimeReporter,
        seccomp_filter: BpfProgramRef,
        maybe_request_size: Option<usize>,
    ) -> Result<()> {
        let mut server = HttpServer::new(path).unwrap_or_else(|e| {
            error!("Error creating the HTTP server: {}", e);
            std::process::exit(vmm::FC_EXIT_CODE_GENERIC_ERROR);
        });
        if let Some(request_size) = maybe_request_size {
            server.set_payload_max_size(request_size);
        }

        // Store process start time metric.
        process_time_reporter.report_start_time();
        // Store process CPU start time metric.
        process_time_reporter.report_cpu_start_time();

        // Load seccomp filters on the API thread.
        // Execution panics if filters cannot be loaded, use --no-seccomp if skipping filters
        // altogether is the desired behaviour.
        if let Err(e) = seccompiler::apply_filter(seccomp_filter) {
            panic!(
                "Failed to set the requested seccomp filters on the API thread: Error: {:?}",
                e
            );
        }

        server.start_server().expect("Cannot start HTTP server");

        loop {
            let request_vec = match server.requests() {
                Ok(vec) => vec,
                Err(e) => {
                    // print request error, but keep server running
                    error!(
                        "API Server error on retrieving incoming request. Error: {}",
                        e
                    );
                    continue;
                }
            };
            for server_request in request_vec {
                let request_processing_start_us =
                    utils::time::get_time_us(utils::time::ClockType::Monotonic);
                server
                    .respond(
                        // Use `self.handle_request()` as the processing callback.
                        server_request.process(|request| {
                            self.handle_request(request, request_processing_start_us)
                        }),
                    )
                    .or_else(|e| {
                        error!("API Server encountered an error on response: {}", e);
                        Ok(())
                    })?;

                let delta_us = utils::time::get_time_us(utils::time::ClockType::Monotonic)
                    - request_processing_start_us;
                debug!("Total previous API call duration: {} us.", delta_us);

                if self.shutdown_flag {
                    server.flush_outgoing_writes();
                    debug!(
                        "/shutdown-internal request received, API server thread now ending itself"
                    );
                    return Ok(());
                }
            }
        }
    }

    /// Handles an API request received through the associated socket.
    pub fn handle_request(
        &mut self,
        request: &Request,
        request_processing_start_us: u64,
    ) -> Response {
        match ParsedRequest::try_from_request(request).map(|r| r.into_parts()) {
            Ok((req_action, mut parsing_info)) => {
                let mut response = match req_action {
                    RequestAction::Sync(vmm_action) => {
                        self.serve_vmm_action_request(vmm_action, request_processing_start_us)
                    }
                    RequestAction::GetMMDS => self.get_mmds(),
                    RequestAction::PatchMMDS(value) => self.patch_mmds(value),
                    RequestAction::PutMMDS(value) => self.put_mmds(value),
                    RequestAction::ShutdownInternal => {
                        self.shutdown_flag = true;
                        Response::new(Version::Http11, StatusCode::NoContent)
                    }
                };
                if let Some(message) = parsing_info.take_deprecation_message() {
                    warn!("{}", message);
                    response.set_deprecation();
                }
                response
            }
            Err(e) => {
                error!("{}", e);
                e.into()
            }
        }
    }

    fn serve_vmm_action_request(
        &mut self,
        vmm_action: Box<VmmAction>,
        request_processing_start_us: u64,
    ) -> Response {
        let metric_with_action = match *vmm_action {
            VmmAction::CreateSnapshot(ref params) => match params.snapshot_type {
                SnapshotType::Full => Some((
                    &METRICS.latencies_us.full_create_snapshot,
                    "create full snapshot",
                )),
                SnapshotType::Diff => Some((
                    &METRICS.latencies_us.diff_create_snapshot,
                    "create diff snapshot",
                )),
            },
            VmmAction::LoadSnapshot(_) => {
                Some((&METRICS.latencies_us.load_snapshot, "load snapshot"))
            }
            VmmAction::Pause => Some((&METRICS.latencies_us.pause_vm, "pause vm")),
            VmmAction::Resume => Some((&METRICS.latencies_us.resume_vm, "resume vm")),
            _ => None,
        };

        self.api_request_sender
            .send(vmm_action)
            .expect("Failed to send VMM message");
        self.to_vmm_fd.write(1).expect("Cannot update send VMM fd");
        let vmm_outcome = *(self.vmm_response_receiver.recv().expect("VMM disconnected"));
        let response = ParsedRequest::convert_to_response(&vmm_outcome);

        if vmm_outcome.is_ok() {
            if let Some((metric, action)) = metric_with_action {
                let elapsed_time_us =
                    update_metric_with_elapsed_time(metric, request_processing_start_us);
                info!("'{}' API request took {} us.", action, elapsed_time_us);
            }
        }
        response
    }

    fn unlock_mmds(&self) -> MutexGuard<'_, Mmds> {
        self.mmds_info
            .lock()
            .expect("Failed to acquire lock on MMDS info")
    }

    fn get_mmds(&self) -> Response {
        let data = self.unlock_mmds().get_data_str();
        ApiServer::json_response(StatusCode::OK, data)
    }

    fn patch_mmds(&self, value: serde_json::Value) -> Response {
        let mmds_response = self.unlock_mmds().patch_data(value);

        match mmds_response {
            Ok(_) => Response::new(Version::Http11, StatusCode::NoContent),
            Err(e) => match e {
                data_store::Error::NotFound
                | data_store::Error::UnsupportedValueType
                | data_store::Error::TokenAuthority(_) => unreachable!(),
                data_store::Error::NotInitialized => ApiServer::json_response(
                    StatusCode::BadRequest,
                    ApiServer::json_fault_message(e.to_string()),
                ),
                data_store::Error::DataStoreLimitExceeded => ApiServer::json_response(
                    StatusCode::PayloadTooLarge,
                    ApiServer::json_fault_message(e.to_string()),
                ),
            },
        }
    }

    fn put_mmds(&self, value: serde_json::Value) -> Response {
        self.unlock_mmds().put_data(value);
        Response::new(Version::Http11, StatusCode::NoContent)
    }

    /// An HTTP response which also includes a body.
    pub(crate) fn json_response<T: Into<String>>(status: StatusCode, body: T) -> Response {
        let mut response = Response::new(Version::Http11, status);
        response.set_body(Body::new(body.into()));
        response
    }

    fn json_fault_message<T: AsRef<str> + serde::Serialize>(msg: T) -> String {
        json!({ "fault_message": msg }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use logger::StoreMetric;
    use micro_http::HttpConnection;
    use mmds::MMDS;
    use utils::tempfile::TempFile;
    use utils::time::ClockType;
    use vmm::builder::StartMicrovmError;
    use vmm::rpc_interface::VmmActionError;
    use vmm::seccomp_filters::{get_filters, SeccompConfig};
    use vmm::vmm_config::instance_info::InstanceInfo;
    use vmm::vmm_config::snapshot::CreateSnapshotParams;

    #[test]
    fn test_error_messages() {
        let e = Error::Io(io::Error::from_raw_os_error(0));
        assert_eq!(
            format!("{}", e),
            format!("IO error: {}", io::Error::from_raw_os_error(0))
        );
        let e = Error::Eventfd(io::Error::from_raw_os_error(0));
        assert_eq!(
            format!("{}", e),
            format!("EventFd error: {}", io::Error::from_raw_os_error(0))
        );
    }

    #[test]
    fn test_error_debug() {
        let e = Error::Io(io::Error::from_raw_os_error(0));
        assert_eq!(
            format!("{:?}", e),
            format!("IO error: {}", io::Error::from_raw_os_error(0))
        );
        let e = Error::Eventfd(io::Error::from_raw_os_error(0));
        assert_eq!(
            format!("{:?}", e),
            format!("EventFd error: {}", io::Error::from_raw_os_error(0))
        );
    }

    #[test]
    fn test_serve_vmm_action_request() {
        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();

        let mut api_server = ApiServer::new(
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
        );
        to_api
            .send(Box::new(Err(VmmActionError::StartMicrovm(
                StartMicrovmError::MissingKernelConfig,
            ))))
            .unwrap();
        let response = api_server.serve_vmm_action_request(Box::new(VmmAction::StartMicroVm), 0);
        assert_eq!(response.status(), StatusCode::BadRequest);

        let start_time_us = utils::time::get_time_us(ClockType::Monotonic);
        assert_eq!(METRICS.latencies_us.pause_vm.fetch(), 0);
        to_api.send(Box::new(Ok(VmmData::Empty))).unwrap();
        let response =
            api_server.serve_vmm_action_request(Box::new(VmmAction::Pause), start_time_us);
        assert_eq!(response.status(), StatusCode::NoContent);
        assert_ne!(METRICS.latencies_us.pause_vm.fetch(), 0);

        assert_eq!(METRICS.latencies_us.diff_create_snapshot.fetch(), 0);
        to_api
            .send(Box::new(Err(VmmActionError::OperationNotSupportedPreBoot)))
            .unwrap();
        let response = api_server.serve_vmm_action_request(
            Box::new(VmmAction::CreateSnapshot(CreateSnapshotParams {
                snapshot_type: SnapshotType::Diff,
                snapshot_path: PathBuf::new(),
                mem_file_path: PathBuf::new(),
                version: None,
            })),
            start_time_us,
        );
        assert_eq!(response.status(), StatusCode::BadRequest);
        // The metric should not be updated if the request wasn't successful.
        assert_eq!(METRICS.latencies_us.diff_create_snapshot.fetch(), 0);

        to_api.send(Box::new(Ok(VmmData::Empty))).unwrap();
        let response = api_server.serve_vmm_action_request(
            Box::new(VmmAction::CreateSnapshot(CreateSnapshotParams {
                snapshot_type: SnapshotType::Diff,
                snapshot_path: PathBuf::new(),
                mem_file_path: PathBuf::new(),
                version: None,
            })),
            start_time_us,
        );
        assert_eq!(response.status(), StatusCode::NoContent);
        assert_ne!(METRICS.latencies_us.diff_create_snapshot.fetch(), 0);
        assert_eq!(METRICS.latencies_us.full_create_snapshot.fetch(), 0);
    }

    #[test]
    fn test_get_mmds() {
        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (_to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();

        let api_server = ApiServer::new(
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
        );

        let response = api_server.get_mmds();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_put_mmds() {
        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (_to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();

        let api_server = ApiServer::new(
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
        );
        let response = api_server.put_mmds(serde_json::Value::String("string".to_string()));
        assert_eq!(response.status(), StatusCode::NoContent);
    }

    #[test]
    fn test_patch_mmds() {
        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (_to_api, vmm_response_receiver) = channel();
        let mmds_info = Arc::new(Mutex::new(Mmds::default()));

        let api_server = ApiServer::new(
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
        );

        // MMDS data store is not yet initialized.
        let response = api_server.patch_mmds(serde_json::Value::Bool(true));
        assert_eq!(response.status(), StatusCode::BadRequest);

        let response = api_server.put_mmds(serde_json::Value::String("string".to_string()));
        assert_eq!(response.status(), StatusCode::NoContent);

        let response = api_server.patch_mmds(serde_json::Value::String(
            "{ \"key\" : \"value\" }".to_string(),
        ));
        assert_eq!(response.status(), StatusCode::NoContent);
    }

    #[test]
    fn test_handle_request() {
        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();

        let mut api_server = ApiServer::new(
            mmds_info,
            api_request_sender,
            vmm_response_receiver,
            to_vmm_fd,
        );

        // Test an Actions request.
        let (mut sender, receiver) = UnixStream::pair().unwrap();
        let mut connection = HttpConnection::new(receiver);
        sender
            .write_all(
                b"PUT /actions HTTP/1.1\r\n\
                Content-Type: application/json\r\n\
                Content-Length: 49\r\n\r\n{ \
                \"action_type\": \"Invalid\", \
                \"payload\": \"string\" \
                }",
            )
            .unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::BadRequest);

        // Test a Get Info request.
        to_api
            .send(Box::new(Ok(VmmData::InstanceInformation(
                InstanceInfo::default(),
            ))))
            .unwrap();
        sender.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::OK);

        // Test a Get Mmds request.
        sender.write_all(b"GET /mmds HTTP/1.1\r\n\r\n").unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::OK);

        // Test a Put Mmds request.
        sender
            .write_all(
                b"PUT /mmds HTTP/1.1\r\n\
                Content-Type: application/json\r\n\
                Content-Length: 2\r\n\r\n{}",
            )
            .unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::NoContent);

        // Test a Patch Mmds request.
        sender
            .write_all(
                b"PATCH /mmds HTTP/1.1\r\n\
                Content-Type: application/json\r\n\
                Content-Length: 2\r\n\r\n{}",
            )
            .unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::NoContent);

        // Test erroneous request.
        sender
            .write_all(
                b"GET /mmds HTTP/1.1\r\n\
                Content-Type: application/json\r\n\
                Content-Length: 2\r\n\r\n{}",
            )
            .unwrap();
        assert!(connection.try_read().is_ok());
        let req = connection.pop_parsed_request().unwrap();
        let response = api_server.handle_request(&req, 0);
        assert_eq!(response.status(), StatusCode::BadRequest);
    }

    #[test]
    fn test_bind_and_run() {
        let mut tmp_socket = TempFile::new().unwrap();
        tmp_socket.remove().unwrap();
        let path_to_socket = tmp_socket.as_path().to_str().unwrap().to_owned();
        let api_thread_path_to_socket = path_to_socket.clone();

        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (_to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();
        let seccomp_filters = get_filters(SeccompConfig::Advanced).unwrap();

        thread::Builder::new()
            .name("fc_api_test".to_owned())
            .spawn(move || {
                ApiServer::new(
                    mmds_info,
                    api_request_sender,
                    vmm_response_receiver,
                    to_vmm_fd,
                )
                .bind_and_run(
                    PathBuf::from(api_thread_path_to_socket),
                    ProcessTimeReporter::new(Some(1), Some(1), Some(1)),
                    seccomp_filters.get("api").unwrap(),
                    None,
                )
                .unwrap();
            })
            .unwrap();

        // Wait for the server to set itself up.
        thread::sleep(Duration::from_millis(10));
        let mut sock = UnixStream::connect(PathBuf::from(path_to_socket)).unwrap();

        // Send a GET mmds request.
        assert!(sock.write_all(b"GET /mmds HTTP/1.1\r\n\r\n").is_ok());
        let mut buf: [u8; 100] = [0; 100];
        assert!(sock.read(&mut buf[..]).unwrap() > 0);

        // Send an erroneous request.
        assert!(sock.write_all(b"OPTIONS / HTTP/1.1\r\n\r\n").is_ok());
        let mut buf: [u8; 100] = [0; 100];
        assert!(sock.read(&mut buf[..]).unwrap() > 0);
    }

    #[test]
    fn test_bind_and_run_with_limit() {
        let mut tmp_socket = TempFile::new().unwrap();
        tmp_socket.remove().unwrap();
        let path_to_socket = tmp_socket.as_path().to_str().unwrap().to_owned();
        let api_thread_path_to_socket = path_to_socket.clone();

        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK).unwrap();
        let (api_request_sender, _from_api) = channel();
        let (_to_api, vmm_response_receiver) = channel();
        let mmds_info = MMDS.clone();
        let seccomp_filters = get_filters(SeccompConfig::Advanced).unwrap();

        thread::Builder::new()
            .name("fc_api_test".to_owned())
            .spawn(move || {
                ApiServer::new(
                    mmds_info,
                    api_request_sender,
                    vmm_response_receiver,
                    to_vmm_fd,
                )
                .bind_and_run(
                    PathBuf::from(api_thread_path_to_socket),
                    ProcessTimeReporter::new(Some(1), Some(1), Some(1)),
                    seccomp_filters.get("api").unwrap(),
                    Some(50),
                )
                .unwrap();
            })
            .unwrap();

        // Wait for the server to set itself up.
        thread::sleep(Duration::from_millis(10));
        let mut sock = UnixStream::connect(PathBuf::from(path_to_socket)).unwrap();

        // Send a GET mmds request.
        assert!(sock
            .write_all(
                b"PUT http://localhost/home HTTP/1.1\r\n\
                  Content-Length: 50000\r\n\r\naaaaaa"
            )
            .is_ok());
        let mut buf: [u8; 265] = [0; 265];
        assert!(sock.read(&mut buf[..]).unwrap() > 0);
        let error_message = b"HTTP/1.1 400 \r\n\
                              Server: Firecracker API\r\n\
                              Connection: keep-alive\r\n\
                              Content-Type: application/json\r\n\
                              Content-Length: 146\r\n\r\n{ \"error\": \"\
                              Request payload with size 50000 is larger than \
                              the limit of 50 allowed by server.\nAll previous \
                              unanswered requests will be dropped.\" }";
        assert_eq!(&buf[..], &error_message[..]);
    }
}
