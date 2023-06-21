# Changelog

## [Unreleased]

### Added

- Added support for custom CPU templates allowing users to adjust vCPU features
  exposed to the guest via CPUID, MSRs and ARM registers.
- Introduced V1N1 static CPU template for ARM to represent Neoverse V1 CPU
  as Neoverse N1.
- Added a `cpu-template-helper` tool for assisting with creating and managing
  custom CPU templates.
- Added support for the `virtio-rng` entropy device. The device is optional. A
  single device can be enabled per VM using the `/entropy` endpoint.

### Changed

- Updated deserialization of `bitmap` for custom CPU templates to allow usage
  of '_' as a separator.
- Set FDP_EXCPTN_ONLY bit (CPUID.7h.0:EBX[6]) and ZERO_FCS_FDS bit
  (CPUID.7h.0:EBX[13]) in Intel's CPUID normalization process.

### Fixed

- Fixed feature flags in T2S CPU template on Intel Ice Lake.
- Fixed CPUID leaf 0xb to be exposed to guests running on AMD host.
- Fixed a performance regression in the jailer logic for closing open file
  descriptors. Related to:
  [#3542](https://github.com/firecracker-microvm/firecracker/issues/3542).
- A race condition that has been identified between the API thread and the VMM
  thread due to a misconfiguration of the `api_event_fd`.
- Fixed CPUID leaf 0x1 to disable perfmon and debug feature on x86 host.
- Fixed passing through cache information from host in CPUID leaf 0x80000006.

## [1.3.0]

### Added

- Introduced T2CL (Intel) and T2A (AMD) CPU templates to provide
  instruction set feature parity between Intel and AMD CPUs when using
  these templates.
- Added Graviton3 support (c7g instance type).

### Changed

- Improved error message when invalid network backend provided.
- Improved TCP throughput by between 5% and 15% (depending on CPU) by using
  scatter-gather I/O in the net device's TX path.
- Upgraded Rust toolchain from 1.64.0 to 1.66.0.
- Made seccompiler output bit-reproducible.

### Fixed

- Fixed feature flags in T2 CPU template on Intel Ice Lake.

## [1.2.0]

### Added

- Added a new CPU template called `T2S`. This exposes the same CPUID as `T2` to
  the Guest and also overwrites the `ARCH_CAPABILITIES` MSR to expose a reduced
  set of capabilities. With regards to hardware vulnerabilities and mitigations,
  the Guest vCPU will apear to look like a Skylake CPU, making it safe to
  snapshot uVMs running on a newer host CPU (Cascade Lake) and restore on a host
  that has a Skylake CPU.
- Added a new CLI option `--metrics-path PATH`. It accepts a file parameter
  where metrics will be sent to.
- Added baselines for m6i.metal and m6a.metal for all long running performance
  tests.
- Releases now include debuginfo files.

### Changed

- Changed the jailer option `--exec-file` to fail if the filename does not
  contain the string `firecracker` to prevent from running non-firecracker
  binaries.
- Upgraded Rust toolchain from 1.52.1 to 1.64.0.
- Switched to specifying our dependencies using caret requirements instead
  of comparison requirements.
- Updated all dependencies to their respective newest versions.

### Fixed

- Made the `T2` template more robust by explicitly disabling additional
  CPUID flags that should be off but were missed initially or that were
  not available in the spec when the template was created.
- Now MAC address is correctly displayed when queried with GET `/vm/config`
  if left unspecified in both pre and post snapshot states.
- Fixed a self-DoS scenario in the virtio-queue code by reporting and
  terminating execution when the number of available descriptors reported
  by the driver is higher than the queue size.
- Fixed the bad handling of kernel cmdline parameters when init arguments were
  provided in the `boot_args` field of the JSON body of the PUT `/boot-source`
  request.
- Fixed a bug on ARM64 hosts where the upper 64bits of the V0-V31 FL/SIMD
  registers were not saved correctly when taking a snapshot, potentially
  leading to data loss. This change invalidates all ARM64 snapshots taken
  with versions of Firecracker <= 1.1.3.
- Improved stability and security when saving CPU MSRs in snapshots.

## [1.1.0]

### Added

- The API `PATCH` methods for `machine-config` can now be used to reset
  the `cpu_template` to `"None"`. Until this change there was no way to
  reset the `cpu_template` once it was set.
- Added a `rebase-snap` tool for rebasing a diff snapshot over a base
  snapshot.
- Mmds version is persisted across snapshot-restore. Snapshot compatibility is
  preserved bidirectionally, to and from a Firecracker version that does not
  support persisting the Mmds version. In such cases, the default V1 option is
  used.
- Added `--mmds-size-limit` for limiting the mmds data store size instead of
  piggy-backing on `--http-api-max-payload-size`. If left unconfigured it
  defaults to the value of `--http-api-max-payload-size`, to provide backwards
  compatibility.
- Added optional `mem_backend` body field in `PUT` requests on `/snapshot/load`.
  This new parameter is an object that defines the configuration of the backend
  responsible for handling memory loading during snapshot restore. The
  `mem_backend` parameter contains `backend_type` and `backend_path` required
  fields. `backend_type` is an enum that can take either `File` or `Uffd` as
  value. Interpretation of `backend_path` field depends on the value of
  `backend_type`. If `File`, then the user must provide the path to file that
  contains the guest memory to be loaded. Otherwise, if `backend_type` is `Uffd`,
  then `backend_path` is the path to a unix domain socket where a custom page
  fault handler process is listening and expecting a UFFD to be sent by
  Firecracker. The UFFD is used to handle the guest memory page faults in the
  separate process.
- Added logging for the snapshot/restore and async block device IO engine
  features to indicate they are in development preview.

### Changed

- The API `PATCH` method for `/machine-config` can be now used to change
  `track_dirty_pages` on aarch64.
- MmdsV2 is now Generally Available.
- MmdsV1 is now deprecated and will be removed in Firecracker v2.0.0.
  Use MmdsV2 instead.
- Deprecated `mem_file_path` body field in `PUT` on `/snapshot/load` request.

### Fixed

- Fixed inconsistency that allowed the start of a microVM from a JSON file
  without specifying the `vcpu_count` and `mem_size_mib` parameters for
  `machine-config` although they are mandatory when configuring via the API.
  Now these fields are mandatory when specifying `machine-config` in the JSON
  file and when using the `PUT` request on `/machine-config`.
- Fixed inconsistency that allowed a user to specify the `cpu_template`
  parameter and set `smt` to `True` in `machine-config` when starting from a
  JSON file on aarch64 even though they are not permitted when using `PUT` or
  `PATCH` in the API. Now Firecracker will return an error on aarch64 if `smt`
  is set to `True` or if `cpu_template` is specified.
- Fixed inconsistent behaviour of the `PUT` method for `/machine-config` that
  would reset the `track_dirty_pages` parameter to `false` if it was not
  specified in the JSON body of the request, but left the `cpu_template`
  parameter intact if it was not present in the request. Now a `PUT` request
  for `/machine-config` will reset all optional parameters (`smt`,
  `cpu_template`, `track_dirty_pages`) to their default values if they are
  not specified in the `PUT` request.
- Fixed incosistency in the swagger definition with the current state of the
  `/vm/config` endpoint.

## [1.0.0]

### Added

- Added jailer option `--parent-cgroup <relative_path>` to allow the placement
  of microvm cgroups in custom cgroup nested hierarchies. The default value is
  `<exec-file>` which is backwards compatible to the behavior before this
  change.
- Added jailer option `--cgroup-version <1|2>` to support running the jailer
  on systems that have cgroup-v2. Default value is `1` which means that if
  `--cgroup-version` is not specified, the jailer will try to create cgroups
  on cgroup-v1 hierarchies only.
- Added `--http-api-max-payload-size` parameter to configure the maximum payload
  size for PUT and PATCH requests.
- Limit MMDS data store size to `--http-api-max-payload-size`.
- Cleanup all environment variables in Jailer.
- Added metrics for accesses to deprecated HTTP and command line API endpoints.
- Added permanent HTTP endpoint for `GET` on `/version` for getting the
  Firecracker version.
- Added `--metadata` parameter to enable MMDS content to be supplied from a file
  allowing the MMDS to be used when using `--no-api` to disable the API server.
- Checksum file for the release assets.
- Added support for custom headers to MMDS requests. Accepted headers are:
  `X-metadata-token`, which accepts a string value that provides a session
  token for MMDS requests; and `X-metadata-token-ttl-seconds`, which
  specifies the lifetime of the session token in seconds.
- Support and validation for host and guest kernel 5.10.
- A [kernel support policy](docs/kernel-policy.md).
- Added `io_engine` to the pre-boot block device configuration.
  Possible values: `Sync` (the default option) or `Async` (only available for
  kernels newer than 5.10.51). The `Async` variant introduces a block device
  engine that uses io_uring for executing requests asynchronously, which is in
  **developer preview** (NOT for production use).
  See `docs/api_requests/block-io-engine.md`.
- Added `block.io_engine_throttled_events` metric for measuring the number of
  virtio events throttled because of the IO engine.
- New optional `version` field to PUT requests towards `/mmds/config` to
  configure MMDS version. Accepted values are `V1` and `V2` and default is
  `V1`. MMDS `V2` is **developer preview only** (NOT for production use) and
  it does not currently work after snapshot load.
- Mandatory `network_interfaces` field to PUT requests towards
  `/mmds/config` which contains a list of network interface IDs capable of
  forwarding packets to MMDS.

### Changed

- Removed the `--node` jailer parameter.
- Deprecated `vsock_id` body field in `PUT`s on `/vsock`.
- Removed the deprecated the `--seccomp-level parameter`.
- `GET` requests to MMDS require a session token to be provided through
  `X-metadata-token` header when using V2.
- Allow `PUT` requests to MMDS in order to generate a session token
  to be used for future `GET` requests when version 2 is used.
- Remove `allow_mmds_requests` field from the request body that attaches network
  interfaces. Specifying interfaces that allow forwarding requests to MMDS is done
  by adding the network interface's ID to the `network_interfaces` field of PUT
  `/mmds/config` request's body.
- Renamed `/machine-config` `ht_enabled` to `smt`.
- `smt` field is now optional on PUT `/machine-config`, defaulting to
  `false`.
- Configuring `smt: true` on aarch64 via the API is forbidden.

### Fixed

- GET `/vm/config` was returning a default config object after restoring from a
  snapshot. It now correctly returns the config of the original microVM, except
  for boot_config and the cpu_template and smt fields of the machine config,
  which are currently lost.
- Fixed incorrect propagation of init parameters in kernel commandline.
  Related to:
  [#2709](https://github.com/firecracker-microvm/firecracker/issues/2709).
- Adapt T2 and C3 CPU templates for kernel 5.10. Firecracker was not previously
  masking some CPU features of the host or emulated by KVM, introduced in more
  recent kernels: `umip`, `vmx`, `avx512_vnni`.
- Fix jailer's cgroup implementation to accept properties that contain multiple
  dots.

## [0.25.0]

### Added

- Added devtool build `--ssh-keys` flag to support fetching from private
  git repositories.
- Added option to configure block device flush.
- Added `--new-pid-ns` flag to the Jailer in order to spawn the Firecracker
  process in a new PID namespace.
- Added API metrics for `GET`, `PUT` and `PATCH` requests on `/mmds` endpoint.
- Added `--describe-snapshot` flag to Firecracker to fetch the data format
  version of a snapshot state file provided as argument.
- Added `--no-seccomp` parameter for disabling the default seccomp filters.
- Added `--seccomp-filter` parameter for supplying user-provided, custom filters.
- Added the `seccompiler-bin` binary that is used to compile JSON seccomp filters
  into serialized BPF for Firecracker consumption.
- Snapshotting support for GICv2 enabled guests.
- Added `devtool install` to deploy built binaries in `/usr/local/bin` or a
  given path.
- Added code logic to send `VIRTIO_VSOCK_EVENT_TRANSPORT_RESET` on snapshot
  creation, when the Vsock device is active. The event will close active
  connections on the guest.
- Added `GET` request on `/vm/config` that provides full microVM configuration
  as a JSON HTTP response.
- Added `--resource-limit` flag to jailer to limit resources such as: number of
  file descriptors allowed at a time (with a default value of 2048) and maximum
  size of files created by the process.

### Changed

- Changed Docker images repository from DockerHub to Amazon ECR.
- Fixed off-by-one error in virtio-block descriptor address validation.
- Changed the `PATCH` request on `/balloon/statistics` to schedule the first
  statistics update immediately after processing the request.
- Deprecated the `--seccomp-level parameter`. It will be removed  in a future
  release. Using it logs a runtime warning.
- Experimental gnu libc builds use empty default seccomp filters, allowing all
  system calls.

### Fixed

- Fixed non-compliant check for the RTC device ensuring a fixed
  4-sized data buffer.
- Unnecessary interrupt assertion was removed from the RTC.
  However, a dummy interrupt is still allocated for snapshot
  compatibility reasons.
- Fixed the SIGPIPE signal handler so Firecracker no longer exits. The signal
  is still recorded in metrics and logs.
- Fixed ballooning API definitions by renaming all fields which mentioned "MB"
  to use "MiB" instead.
- Snapshot related host files (vm-state, memory, block backing files) are now
  flushed to their backing mediums as part of the CreateSnapshot operation.
- Fixed the SSBD mitigation not being enabled on `aarch64` with the provided
  `prod-host-setup.md`.
- Fixed the balloon statistics not working after a snapshot restore event.
- The `utc_timestamp_ms` now reports the timestamp in ms from the UTC UNIX
  Epoch, as the name suggests. It was previously using a monotonic clock with
  an undefined starting point.

## [0.24.0]

### Added

- Added optional `resume_vm` field to `/snapshot/load` API call.
- Added support for block rate limiter PATCH.
- Added devtool test `-c|--cpuset-cpus` flag for cpus confinement when tests
  run.
- Added devtool test `-m|--cpuset-mems` flag for memory confinement when tests
  run.
- Added the virtio traditional memory ballooning device.
- Added a mechanism to handle vCPU/VMM errors that result in process termination.
- Added incremental guest memory snapshot support.
- Added aarch64 snapshot support.

### Changed

- Change the information provided in `DescribeInstance` command to provide microVM
  state information (Not started/Running/Paused) instead of whether it's
  started or not.
- Removed the jailer `--extra-args` parameter. It was a noop, having been
  replaced by the `--` separator for extra arguments.
- Changed the output of the `--version` command line parameter to include a list
  of supported snapshot data format versions for the firecracker binary.
- Increased the maximum number of virtio devices from 11 to 19.
- Added a new check that prevents creating v0.23 snapshots when more than 11
  devices are attached.
- If the stdout buffer is full and non-blocking, the serial writes no longer block.
  Any new bytes will be lost, until the buffer is freed. The device also logs these
  errors and increments the `uart.error_count` metric for each lost byte.

### Fixed

- Fixed inconsistency in YAML file InstanceInfo definition

## [0.23.0]

### Added

- Added metric for throttled block device events.
- Added metrics for counting rate limiter throttling events.
- Added metric for counting MAC address updates.
- Added metrics for counting TAP read and write errors.
- Added metrics for counting RX and TX partial writes.
- Added metrics that measure the duration of pausing and resuming the microVM,
  from the VMM perspective.
- Added metric for measuring the duration of the last full/diff snapshot created,
  from the VMM perspective.
- Added metric for measuring the duration of loading a snapshot, from the VMM
  perspective.
- Added metrics that measure the duration of pausing and resuming the microVM,
  from the API (user) perspective.
- Added metric for measuring the duration of the last full/diff snapshot created,
  from the API (user) perspective.
- Added metric for measuring the duration of loading a snapshot, from the API
  (user) perspective.
- Added `track_dirty_pages` field to `machine-config`. If enabled, Firecracker
  can create incremental guest memory snapshots by saving the dirty guest pages
  in a sparse file.
- Added a new API call, `PATCH /vm`, for changing the microVM state (to
  `Paused` or `Resumed`).
- Added a new API call, `PUT /snapshot/create`, for creating a full or diff snapshot.
- Added a new API call, `PUT /snapshot/load`, for loading a snapshot.
- Added new jailer command line argument `--cgroup` which allow the user to
  specify the cgroups that are going to be set by the Jailer.
- Added full support for AMD CPUs (General Availability). More details
  [here](README.md#supported-platforms).

### Fixed

- Boot time on AMD achieves the desired performance (i.e under 150ms).

### Changed

- The logger `level` field is now case-insensitive.
- Disabled boot timer device after restoring a snapshot.
- Enabled boot timer device only when specifically requested, by using the
  `--boot-timer` dedicated cmdline parameter.
- firecracker and jailer `--version` now gets updated on each devtool
  build to the output of `git describe --dirty`, if the git repo is available.
- MicroVM process is only attached to the cgroups defined by using `--cgroups`
  or the ones defined indirectly by using `--node`.
- Changed `devtool build` to build jailer binary for `musl` only targets. Building
  jailer binary for `non-musl` targets have been removed.

## [0.22.0]

### Added

- Added a new API call, `PUT /metrics`, for configuring the metrics system.
- Added `app_name` field in InstanceInfo struct for storing the application
  name.
- New command-line parameters for `firecracker`, named `--log-path`,
  `--level`, `--show-level` and `--show-log-origin` that can be used
  for configuring the Logger when starting the process. When using
  this method for configuration, only `--log-path` is mandatory.
- Added a [guide](docs/devctr-image.md) for updating the dev container image.
- Added a new API call, `PUT /mmds/config`, for configuring the
  `MMDS` with a custom valid link-local IPv4 address.
- Added experimental JSON response format support for MMDS guest applications
  requests.
- Added metrics for the vsock device.
- Added `devtool strip` command which removes debug symbols from the release
  binaries.
- Added the `tx_malformed_frames` metric for the virtio net device, emitted
  when a TX frame missing the VNET header is encountered.

### Fixed

- Added `--version` flag to both Firecracker and Jailer.
- Return `405 Method Not Allowed` MMDS response for non HTTP `GET` MMDS
  requests originating from guest.
- Fixed folder permissions in the jail (#1802).
- Any number of whitespace characters are accepted after ":" when parsing HTTP
  headers.
- Potential panic condition caused by the net device expecting to find a VNET
  header in every frame.
- Potential crash scenario caused by "Content-Length" HTTP header field
  accepting negative values.
- Fixed #1754 - net: traffic blocks when running ingress UDP performance tests
  with very large buffers.

### Changed

- Updated CVE-2019-3016 mitigation information in
  [Production Host Setup](docs/prod-host-setup.md)
- In case of using an invalid JSON as a 'config-file' for Firecracker,
  the process will exit with return code 152.
- Removed the `testrun.sh` wrapper.
- Removed `metrics_fifo` field from the logger configuration.
- Renamed `log_fifo` field from LoggerConfig to `log_path` and
  `metrics_fifo` field from MetricsConfig to `metrics_path`.
- `PATCH /drives/{id}` only allowed post-boot. Use `PUT` for pre-boot
  updates to existing configurations.
- `PATCH /network-interfaces/{id}` only allowed post-boot. Use `PUT` for
  pre-boot updates to existing configurations.
- Changed returned status code from `500 Internal Server Error` to
  `501 Not Implemented`, for queries on the MMDS endpoint in IMDS format, when
  the requested resource value type is unsupported.
- Allowed the MMDS data store to be initialized with all supported JSON types.
  Retrieval of these values within the guest, besides String, Array, and
  Dictionary, is only possible in JSON mode.
- `PATCH` request on `/mmds` before the data store is initialized returns
  `403 BadRequest`.
- Segregated MMDS documentation in MMDS design documentation and MMDS user
  guide documentation.

## [0.21.0]

### Added

- Support for booting with an initial RAM disk image. This image can be
  specified through the new `initrd_path` field of the `/boot-source` API
  request.

### Fixed

- Fixed #1469 - Broken GitHub location for Firecracker release binary.
- The jailer allows changing the default api socket path by using the extra
  arguments passed to firecracker.
- Fixed #1456 - Occasional KVM_EXIT_SHUTDOWN and bad syscall (14) during
  VM shutdown.
- Updated the production host setup guide with steps for addressing
  CVE-2019-18960.
- The HTTP header parsing is now case insensitive.
- The `put_api_requests` and `patch_api_requests` metrics for net devices were
  un-swapped.

### Changed

- Removed redundant `--seccomp-level` jailer parameter since it can be
  simply forwarded to the Firecracker executable using "end of command
  options" convention.
- Removed `memory.dirty_pages` metric.
- Removed `options` field from the logger configuration.
- Decreased release binary size by ~15%.
- Changed default API socket path to `/run/firecracker.socket`. This path
  also applies when running with the jailer.
- Disabled KVM dirty page tracking by default.
- Removed redundant RescanBlockDevice action from the /actions API.
  The functionality is available through the PATCH /drives API.
  See `docs/api_requests/patch-block.md`.

## [0.20.0]

### Added

- Added support for GICv2.

### Fixed

- Fixed CVE-2019-18960 - Fixed a logical error in bounds checking performed
  on vsock virtio descriptors.
- Fixed #1283 - Can't start a VM in AARCH64 with vcpus number more than 16.
- Fixed #1088 - The backtrace are printed on `panic`, no longer causing a
  seccomp fault.
- Fixed #1375 - Change logger options type from `Value` to `Vec<LogOption>` to
  prevent potential unwrap on None panics.
- Fixed #1436 - Raise interrupt for TX queue used descriptors
- Fixed #1439 - Prevent achieving 100% cpu load when the net device rx is
  throttled by the ratelimiter
- Fixed #1437 - Invalid fields in rate limiter related API requests are
  now failing with a proper error message.
- Fixed #1316 - correctly determine the size of a virtio device backed
  by a block device.
- Fixed #1383 - Log failed api requests.

### Changed

- Decreased release binary size by 10%.

## [0.19.0]

### Added

- New command-line parameter for `firecracker`, named `--no-api`, which
  will disable the API server thread. If set, the user won't be able to send
  any API requests, neither before, nor after the vm has booted. It must be
  paired with `--config-file` parameter. Also, when API server is disabled,
  MMDS is no longer available now.
- New command-line parameter for `firecracker`, named `--config-file`, which
  represents the path to a file that contains a JSON which can be used for
  configuring and starting a microVM without sending any API requests.
- The jailer adheres to the "end of command options" convention, meaning
  all parameters specified after `--` are forwarded verbatim to Firecracker.
- Added `KVM_PTP` support to the recommended guest kernel config.
- Added entry in FAQ.md for Firecracker Guest timekeeping.

### Changed

- Vsock API call: `PUT /vsocks/{id}` changed to `PUT /vsock` and no longer
  appear to support multiple vsock devices. Any subsequent calls to this API
  endpoint will override the previous vsock device configuration.
- Removed unused 'Halting' and 'Halted' instance states.
- Vsock host-initiated connections now implement a trivial handshake protocol.
  See the [vsock doc](docs/vsock.md#host-initiated-connections) for details.
  Related to:
  [#1253](https://github.com/firecracker-microvm/firecracker/issues/1253),
  [#1432](https://github.com/firecracker-microvm/firecracker/issues/1432),
  [#1443](https://github.com/firecracker-microvm/firecracker/pull/1443)

### Fixed

- Fixed serial console on aarch64 (GitHub issue #1147).
- Upon panic, the terminal is now reset to canonical mode.
- Explicit error upon failure of vsock device creation.
- The failure message returned by an API call is flushed in the log FIFOs.
- Insert virtio devices in the FDT in order of their addresses sorted from
  low to high.
- Enforce the maximum length of the network interface name to be 16 chars as
  specified in the Linux Kernel.
- Changed the vsock property `id` to `vsock_id` so that the API client can be
  successfully generated from the swagger definition.

## [0.18.0]

### Added

- New device: virtio-vsock, backed by Unix domain sockets (GitHub issue #650).
  See `docs/vsock.md`.

### Changed

- No error is thrown upon a flush metrics intent if logger has not been
  configured.

### Fixed

- Updated the documentation for integration tests.
- Fixed high CPU usage before guest network interface is brought up (GitHub
  issue #1049).
- Fixed an issue that caused the wrong date (month) to appear in the log.
- Fixed a bug that caused the seccomp filter to reject legit syscalls in some
  rare cases (GitHub issue #1206).
- Docs: updated the production host setup guide.
- Docs: updated the rootfs and kernel creation guide.

### Removed

- Removed experimental support for vhost-based vsock devices.

## [0.17.0]

### Added

- New API call: `PATCH /machine-config/`, used to update VM configuration,
  before the microVM boots.
- Added an experimental swagger definition that includes the specification for
  the vsock API call.
- Added a signal handler for `SIGBUS` and `SIGSEGV` that immediately terminates
  the process upon intercepting the signal.
- Added documentation for signal handling utilities.
- Added [alpha] aarch64 support.
- Added metrics for successful read and write operations of MMDS, Net and Block
  devices.

### Changed

- `vcpu_count`, `mem_size_mib` and `ht_enabled` have been changed to be mandatory
  for `PUT` requests on `/machine-config/`.
- Disallow invalid seccomp levels by exiting with error.

### Fixed

- Incorrect handling of bind mounts within the jailed rootfs.
- Corrected the guide for `Alpine` guest setup.

## [0.16.0]

### Added

- Added [alpha] AMD support.
- New `devtool` command: `prepare_release`. This updates the Firecracker
  version, crate dependencies and credits in preparation for a new release.
- New `devtool` command: `tag`. This creates a new git tag for the specified
  release number, based on the changelog contents.
- New doc section about building with glibc.

### Changed

- Dropped the JSON-formatted `context` command-line parameter from Firecracker
  in favor of individual classic command-line parameters.
- When running with `jailer` the location of the API socket has changed to
  `<jail-root-path>/api.socket` (API socket was moved _inside_ the jail).
- `PUT` and `PATCH` requests on `/mmds` with data containing any value type
  other than `String`, `Array`, `Object` will returns status code 400.
- Improved multiple error messages.
- Removed all kernel modules from the recommended kernel config.

### Fixed

- Corrected the seccomp filter when building with glibc.

### Removed

- Removed the `seccomp.bad_syscalls` metric.

## [0.15.2]

### Fixed

- Corrected the conditional compilation of the seccomp rule for `madvise`.

## [0.15.1]

### Fixed

- A `madvise` call issued by the `musl` allocator was added to the seccomp
  allow list to prevent Firecracker from terminating abruptly when allocating
  memory in certain conditions.

## [0.15.0]

### Added

- New API action: SendCtrlAltDel, used to initiate a graceful shutdown,
  if the guest has driver support for i8042 and AT Keyboard. See
  [the docs](docs/api_requests/actions.md#sendctrlaltdel) for details.
- New metric counting the number of egress packets with a spoofed MAC:
  `net.tx_spoofed_mac_count`.
- New API call: `PATCH /network-interfaces/`, used to update the rate limiters
  on a network interface, after the start of a microVM.

### Changed

- Added missing `vmm_version` field to the InstanceInfo API swagger
  definition, and marked several other mandatory fields as such.
- New default command line for guest kernel:
  `reboot=k panic=1 pci=off nomodules 8250.nr_uarts=0
  i8042.noaux i8042.nomux i8042.nopnp i8042.dumbkbd`.

### Fixed

- virtio-blk: VIRTIO_BLK_T_FLUSH now working as expected.
- Vsock devices can be attached when starting Firecracker using the jailer.
- Vsock devices work properly when seccomp filtering is enabled.

## [0.14.0]

### Added

- Documentation for development environment setup on AWS in
  `dev-machine-setup.md`.
- Documentation for microVM networking setup in `docs/network-setup.md`.
- Limit the maximum supported vCPUs to 32.

### Changed

- Log the app version when the `Logger` is initialized.
- Pretty print panic information.
- Firecracker terminates with exit code 148 when a syscall which is not
  present in the allow list is intercepted.

### Fixed

- Fixed build with the `vsock` feature.

## [0.13.0]

### Added

- Documentation for Logger API Requests in `docs/api_requests/logger.md`.
- Documentation for Actions API Requests in `docs/api_requests/actions.md`.
- Documentation for MMDS in `docs/mmds.md`.
- Flush metrics on request via a PUT `/actions` with the `action_type`
  field set to `FlushMetrics`.

### Changed

- Updated the swagger definition of the `Logger` to specify the required fields
  and provide default values for optional fields.
- Default `seccomp-level` is `2` (was previously 0).
- API Resource IDs can only contain alphanumeric characters and underscores.

### Fixed

- Seccomp filters are now applied to all Firecracker threads.
- Enforce minimum length of 1 character for the jailer ID.
- Exit with error code when starting the jailer process fails.

### Removed

- Removed `InstanceHalt` from the list of possible actions.

## [0.12.0]

### Added

- The `/logger` API has a new field called `options`. This is an array of
  strings that specify additional logging configurations. The only supported
  value is `LogDirtyPages`.
- When the `LogDirtyPages` option is configured via `PUT /logger`, a new metric
  called `memory.dirty_pages` is computed as the number of pages dirtied by the
  guest since the last time the metric was flushed.
- Log messages on both graceful and forceful termination.
- Availability of the list of dependencies for each commit inside the code
  base.
- Documentation on vsock experimental feature and host setup recommendations.

### Changed

- `PUT` requests on `/mmds` always return 204 on success.
- `PUT` operations on `/network-interfaces` API resources no longer accept
  the previously required `state` parameter.
- The jailer starts with `--seccomp-level=2` (was previously 0) by default.
- Log messages use `anonymous-instance` as instance id if none is specified.

### Fixed

- Fixed crash upon instance start on hosts without 1GB huge page support.
- Fixed "fault_message" inconsistency between Open API specification and code
  base.
- Ensure MMDS compatibility with C5's IMDS implementation.
- Corrected the swagger specification to ensure `OpenAPI 2.0` compatibility.

## [0.11.0]

### Added

- Apache-2.0 license
- Docs:
  - [charter](CHARTER.md)
  - [contribution guildelines](CONTRIBUTE.md)
  - [design](docs/design.md)
  - [getting started guide](docs/getting-started.md)
  - [security policy](SECURITY.md)
  - [specifications](SPECIFICATION.md)
- **Experimental** vhost-based vsock implementation.

### Changed

- Improved MMDS network stack performance.
- If the logging system is not yet initialized (via `PUT /logger`), log events
  are now sent to stdout/stderr.
- Moved the `instance_info_fails` metric under `get_api_requests`
- Improved [readme](README.md) and added links to more detailed information,
  now featured in subject-specific docs.

### Fixed

- Fixed bug in the MMDS network stack, that caused some RST packets to be sent
  without a destination.
- Fixed bug in `PATCH /drives`, whereby the ID in the path was not checked
  against the ID in the body.

## [0.10.1]

### Fixed

- The Swagger definition was corrected.

## [0.10.0]

### Added

- Each Firecracker process has an associated microVM Metadata Store (MMDS). Its
  contents can be configured using the `/mmds` API resource.

### Changed

- The boot source is specified only with the `kernel_image_path` and
  the optional parameter `boot_args`. All other fields are removed.
- The `path_on_host` property in the drive specification is now marked as
  *mandatory*.
- PATCH drive only allows patching/changing the `path_on_host` property.
- All PUT and PATCH requests return the status code 204.
- CPUID brand string (aka model name) now includes the host CPU frequency.
- API requests which add guest network interfaces have an additional parameter,
  `allow_mmds_requests` which defaults to `false`.
- Stopping the guest (e.g. using the `reboot` command) also terminates the
  Firecracker process. When the Firecracker process ends for any reason,
  (other than `kill -9`), metrics are flushed at the very end.
- On startup `jailer` closes all inherited file descriptors based on
  `sysconf(_SC_OPEN_MAX)` except input, output and error.
- The microVM ID prefixes each Firecracker log line. This ID also appears
  in the process `cmdline` so it's now possible to `ps | grep <ID>` for it.

## [0.9.0]

### Added

- Seccomp filtering is configured via the `--seccomp-level` jailer parameter.
- Firecracker logs the starting addresses of host memory areas provided as
  guest memory slots to KVM.
- The metric `panic_count` gets incremented to signal that a panic has
  occurred.
- Firecracker logs a backtrace when it crashes following a panic.
- Added basic instrumentation support for measuring boot time.

### Changed

- `StartInstance` is a synchronous API request (it used to be an asynchronous
  request).

### Fixed

- Ensure that fault messages sent by the API have valid JSON bodies.
- Use HTTP response code 500 for internal Firecracker errors, and 400 for user
  errors on InstanceStart.
- Serialize the machine configuration fields to the correct data types (as
  specified in the Swagger definition).
- NUMA node assignment is properly enforced by the jailer.
- The `is_root_device` and `is_read_only` properties are now marked as required
  in the Swagger definition of `Drive` object properties.

### Removed

- `GET` requests on the `/actions` API resource are no longer supported.
- The metrics associated with asynchronous actions have been removed.
- Remove the `action_id` parameter for `InstanceStart`, both from the URI and
  the JSON request body.

## [0.8.0]

### Added

- The jailer can now be configured to enter a preexisting network namespace,
  and to run as a daemon.
- Enabled PATCH operations on `/drives` resources.

## Changed

- The microVM `id` supplied to the jailer may now contain alphanumeric
  characters and hyphens, up to a maximum length of 64 characters.
- Replaced the `permissions` property of `/drives` resources with a boolean.
- Removed the `state` property of `/drives` resources.

## [0.7.0]

### Added

- Rate limiting functionality allows specifying an initial one time
  burst size.
- Firecracker can now boot from an arbitrary boot partition by specifying
  its unique id in the driver's API call.
- Block device rescan is triggered via a PUT `/actions` with the drive ID in
  the action body's `payload` field and the `action_type` field set to
  `BlockDeviceRescan`.

### Changed

- Removed `noapic` from the default guest kernel command line.
- The `action_id` parameter is no longer required for synchronous PUT requests
  to `/actions`.
- PUT requests are no longer allowed on `/drives` resources after the guest
  has booted.

### Fixed

- Fixed guest instance kernel loader to accelerate vCPUs launch and
  consequently guest kernel boot.
- Fixed network emulation to improve IO performance.

## [0.6.0]

### Added

- Firecracker uses two different named pipes to record human readable logs and
  metrics, respectively.

### Changed

- Seccomp filtering can be enabled via setting the `USE_SECCOMP` environment
  variable.
- It is possible to supply only a partial specification when attaching a rate
  limiter (i.e. just the bandwidth or ops parameter).
- Errors related to guest network interfaces are now more detailed.

### Fixed

- Fixed a bug that was causing Firecracker to panic whenever a `PUT` request
  was sent on an existing network interface.
- The `id` parameter of the `jailer` is required to be an RFC 4122-compliant
  UUID.
- Fixed an issue which caused the network RX rate limiter to be more
  restrictive than intended.
- API requests which contain unknown fields will generate an error.
- Fixed an issue related to high CPU utilization caused by improper `KVM PIT`
  configuration.
- It is now possible to create more than one network tun/tap interface inside a
  jailed Firecracker.

## [0.5.0]

### Added

- Added metrics for API requests, VCPU and device actions for the serial
  console (`UART`), keyboard (`i8042`), block and network devices. Metrics are
  logged every 60 seconds.
- A CPU features template for C3 is available, in addition to the one for T2.
- Seccomp filters restrict Firecracker from calling any other system calls than
  the minimum set it needs to function properly. The filters are enabled by
  setting the `USE_SECCOMP` environment variable to 1 before running
  Firecracker.
- Firecracker can be started by a new binary called `jailer`. The jailer takes
  as command line arguments a unique ID, the path to the Firecracker binary,
  the NUMA node that Firecracker will be assigned to and a `uid` and `gid` for
  Firecracker to run under. It sets up a `chroot` environment and a `cgroup`,
  and calls exec to morph into Firecracker.

### Changed

- In case of failure, the metrics and the panic location are logged before
  aborting.
- Metric values are reset with every flush.
- `CPUTemplate` is now called `CpuTemplate` in order to work seamlessly with
  the swagger code generator for Go.
- `firecracker-beta.yaml` is now called `firecracker.yaml`.

### Fixed

- Handling was added for several untreated KVM exit scenarios, which could have
  led to panic.
- Fixed a bug that caused Firecracker to crash when attempting to disable the
  `IA32_DEBUG_INTERFACE MSR` flag in the T2 CPU features.

### Removed

- Removed a leftover file generated by the logger unit tests.
- Removed `firecracker-v1.0.yaml`.

## [0.4.0]

### Added

- The CPU Template can be set with an API call on `PUT /machine-config`. The
  only available template is T2.
- Hyperthreading can be enabled/disabled with an API call on
  `PUT /machine-config`. By default, hyperthreading is disabled.
- Added boot time performance test (`tests/performance/test_boottime.py`).
- Added Rate Limiter for VirtIO/net and VirtIO/net devices. The Rate Limiter
  uses two token buckets to limit rate on bytes/s and ops/s. The rate limiter
  can be (optionally) configured per drive with a `PUT` on `/drives/{drive_id}`
  and per network interface with a `PUT` on `/network-interface/{iface_id}`.
- Implemented pre-boot PUT updates for `/boot-source`, `/drives`,
  `/network-interfaces` and `/vsock`.
- Added integration tests for `PUT` updates.

### Changed

- Moved the API definition (`swagger/firecracker-beta.yaml`) to the
  `api_server` crate.
- Removed `"console=ttyS0"` and added `"8250.nr_uarts=0"` to the default kernel
  command line to decrease the boot time.
- Changed the CPU topology to have all logical CPUs on a single socket.
- Removed the upper bound on CPU count as with musl there is no good way to get
  the total number of logical processors on a host.
- Build time tests now print the full output of commands.
- Disabled the Performance Monitor Unit and the Turbo Boost.
- Check the expected KVM capabilities before starting the VM.
- Logs now have timestamps.

### Fixed

- `testrun.sh` can run on platforms with more than one package manager by
  setting the package manager via a command line parameter (`-p`).
- Allow correct set up of multiple network-interfaces with auto-generated MAC.
- Fixed sporadic bug in VirtIO which was causing lost packages.
- Don't allow `PUT` requests with empty body on `/machine-config`.
- Deny `PUT` operations after the microvm boots (exception: the temporarily fix
  for live resize of block devices).

### Removed

- Removed examples crate. This used to have a Python example of starting
  Firecracker. This is replaced by `test_api.py` integration tests.
- Removed helper scripts for getting coverage and coding style errors. These
  were replaced by `test_coverage.py` and `test_style.py` test integration
  tests.
- Removed `--vmm-no-api` command line option. Firecracker can only be started
  via the API.

## [0.3.0]

### Added

- Users can interrogate the Machine Configuration (i.e. vcpu count and memory
  size) using a `GET` request on `/machine-config`.
- The logging system can be configured through the API using a `PUT` on
  `/logger`.
- Block devices support live resize by calling `PUT` with the same parameters
  as when the block was created.
- Release builds have Link Time Optimization (LTO) enabled.
- Firecracker is built with `musl`, resulting in a statically linked binary.
- More in-tree integration tests were added as part of the continuous
  integration system.

### Changed

- The vcpu count is enforced to `1` or an even number.
- The Swagger definition of rate limiters was updated.
- Syslog-enabled logs were replaced with a host-file backed mechanism.

### Fixed

- The host topology of the CPU and the caches is not leaked into the microvm
  anymore.
- Boot time was improved by advertising the availability of the TSC deadline
  timer.
- Fixed an issue which prevented Firecracker from working on 4.14 (or newer)
  host kernels.
- Specifying the MAC address for an interface through the API is optional.

### Removed

- Removed support for attaching vsock devices.
- Removed support for building Firecracker with glibc.

## [0.2.0]

### Added

- Users can now interrogate Instance Information (currently just instance
  state) through the API.

### Changed

- Renamed `api/swagger/all.yaml` to `api/swagger/firecracker-v1.0.yaml` which
  specifies targeted API support for Firecracker v1.0.
- Renamed `api/swagger/firecracker-v0.1.yaml` to
  `api/swagger/firecracker-beta.yaml` which specifies the currently supported
  API.
- Users can now enforce that an emulated block device is read-only via the API.
  To specify whether a block device is read-only or read-write, an extra
  "permissions" field was added to the Drive definition in the API. The root
  filesystem is automatically mounted in the guest OS as `ro`/`rw` according to
  the specified "permissions". It's the responsibility of the user to mount any
  other read-only block device as such within the guest OS.
- Users can now stop the guest VM using the API. Actions of type `InstanceHalt`
  are now supported via the API.

### Fixed

- Added support for `getDeviceID()` in `virtIO-block`. Without this, the guest
  Linux kernel would complain at boot time that the operation is unsupported.
- `stdin` control is returned to the Firecracker process when guest VM is
  inactive. Raw mode `stdin` is forwarded to the guest OS when guest VM is
  running.

### Removed

- Removed `api/swagger/actions.yaml`.
- Removed `api/swagger/devices.yaml`.
- Removed `api/swagger/firecracker-mvp.yaml`.
- Removed `api/swagger/limiters.yaml`.

## [0.1.1]

### Changed

- Users can now specify the MAC address of a guest network interface via the
  `PUT` network interface API request. Previously, the guest MAC address
  parameter was ignored.

### Fixed

- Fixed a guest memory allocation issue, which previously led to a potentially
  significant memory chunk being wasted.
- Fixed an issue which caused compilation problems, due to a compatibility
  breaking transitive dependency in the tokio suite of crates.

## [0.1.0]

### Added

- One-process virtual machine manager (one Firecracker per microVM).
- RESTful API running on a unix socket. The API supported by v0.1 can be found
  at `api/swagger/firecracker-v0.1.yaml`.
- Emulated keyboard (`i8042`) and serial console (`UART`). The microVM serial
  console input and output are connected to those of the Firecracker process
  (this allows direct console access to the guest OS).
- The capability of mapping an existing host tun-tap device as a VirtIO/net
  device into the microVM.
- The capability of mapping an existing host file as a GirtIO/block device into
  the microVM.
- The capability of creating a VirtIO/vsock between the host and the microVM.
- Default demand fault paging & CPU oversubscription.
