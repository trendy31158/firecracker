# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Define some helpers methods to create microvms from artifacts."""

import json
import os
import tempfile
from pathlib import Path
from conftest import init_microvm
from framework.artifacts import Artifact, DiskArtifact, Snapshot, SnapshotType
import framework.utils as utils
import host_tools.logging as log_tools


class MicrovmBuilder:
    """Build fresh microvms or restore from snapshot."""

    ROOT_PREFIX = "fctest-"

    def __init__(self, bin_cloner_path):
        """Initialize microvm root and cloning binary."""
        self.bin_cloner_path = bin_cloner_path
        self.init_root_path()

    def init_root_path(self):
        """Initialize microvm root path."""
        self.root_path = tempfile.mkdtemp(MicrovmBuilder.ROOT_PREFIX)

    def build(self,
              kernel: Artifact,
              disks: [DiskArtifact],
              ssh_key: Artifact,
              config: Artifact,
              enable_diff_snapshots=False):
        """Build a fresh microvm."""
        self.init_root_path()

        vm = init_microvm(self.root_path, self.bin_cloner_path)

        # Link the microvm to kernel, rootfs, ssh_key artifacts.
        vm.kernel_file = kernel.local_path()
        vm.rootfs_file = disks[0].local_path()

        # Download ssh key into the microvm root.
        ssh_key.download(self.root_path)
        vm.ssh_config['ssh_key_path'] = ssh_key.local_path()
        os.chmod(vm.ssh_config['ssh_key_path'], 0o400)

        # Start firecracker.
        vm.spawn()

        with open(config.local_path()) as microvm_config_file:
            microvm_config = json.load(microvm_config_file)

        # Apply the microvm artifact configuration
        vm.basic_config(vcpu_count=int(microvm_config['vcpu_count']),
                        mem_size_mib=int(microvm_config['mem_size_mib']),
                        ht_enabled=bool(microvm_config['ht_enabled']),
                        track_dirty_pages=enable_diff_snapshots,
                        boot_args='console=ttyS0 reboot=k panic=1')
        return vm

    # This function currently returns the vm and a metrics_fifo which
    # is needed by the performance integration tests.
    # TODO: Move all metrics functionality to microvm (encapsulating the fifo)
    # so we do not need to move it around polluting the code.
    def build_from_snapshot(self,
                            snapshot: Snapshot,
                            host_ip=None,
                            guest_ip=None,
                            netmask_len=None,
                            resume=False,
                            # Enable incremental snapshot capability.
                            enable_diff_snapshots=False):
        """Build a microvm from a snapshot artifact."""
        self.init_root_path()
        vm = init_microvm(self.root_path, self.bin_cloner_path)

        vm.spawn(log_level='Info')

        metrics_file_path = os.path.join(vm.path, 'metrics.log')
        metrics_fifo = log_tools.Fifo(metrics_file_path)
        response = vm.metrics.put(
            metrics_path=vm.create_jailed_resource(metrics_fifo.path)
        )
        assert vm.api_session.is_status_no_content(response.status_code)

        # Hardlink all the snapshot files into the microvm jail.
        jailed_mem = vm.create_jailed_resource(snapshot.mem)
        jailed_vmstate = vm.create_jailed_resource(snapshot.vmstate)
        assert len(snapshot.disks) > 0, "Snapshot requiures at least one disk."
        _jailed_disks = []
        for disk in snapshot.disks:
            _jailed_disks.append(vm.create_jailed_resource(disk))
        vm.ssh_config['ssh_key_path'] = snapshot.ssh_key

        if host_ip is not None:
            vm.create_tap_and_ssh_config(host_ip=host_ip,
                                         guest_ip=guest_ip,
                                         netmask_len=netmask_len,
                                         tapname="tap0")

        response = vm.snapshot_load.put(mem_file_path=jailed_mem,
                                        snapshot_path=jailed_vmstate,
                                        diff=enable_diff_snapshots)

        assert vm.api_session.is_status_no_content(response.status_code)

        if resume:
            # Resume microvm
            response = vm.vm.patch(state='Resumed')
            assert vm.api_session.is_status_no_content(response.status_code)

        # Return a resumed microvm.
        return vm, metrics_fifo


class SnapshotBuilder:  # pylint: disable=too-few-public-methods
    """Create a snapshot from a running microvm."""

    def __init__(self, microvm):
        """Initialize the snapshot builder."""
        self._microvm = microvm

    def create(self,
               disks,
               ssh_key: Artifact,
               snapshot_type: SnapshotType = SnapshotType.FULL):
        """Create a Snapshot object from a microvm and artifacts."""
        # Disable API timeout as the APIs for snapshot related procedures
        # take longer.
        self._microvm.api_session.untime()
        chroot_path = self._microvm.jailer.chroot_path()
        snapshot_dir = os.path.join(chroot_path, "snapshot")
        Path(snapshot_dir).mkdir(parents=True, exist_ok=True)
        cmd = 'chown {}:{} {}'.format(self._microvm.jailer.uid,
                                      self._microvm.jailer.gid,
                                      snapshot_dir)
        utils.run_cmd(cmd)
        self._microvm.pause_to_snapshot(
            mem_file_path="/snapshot/vm.mem",
            snapshot_path="/snapshot/vm.vmstate",
            diff=snapshot_type == SnapshotType.DIFF)

        # Create a copy of the ssh_key artifact.
        ssh_key_copy = ssh_key.copy()
        mem_path = os.path.join(snapshot_dir, "vm.mem")
        vmstate_path = os.path.join(snapshot_dir, "vm.vmstate")
        return Snapshot(mem=mem_path,
                        vmstate=vmstate_path,
                        # TODO: To support more disks we need to figure out a
                        # simple and flexible way to store snapshot artifacts
                        # in S3. This should be done in a PR where we add tests
                        # that resume from S3 snapshot artifacts.
                        disks=disks,
                        ssh_key=ssh_key_copy.local_path())
