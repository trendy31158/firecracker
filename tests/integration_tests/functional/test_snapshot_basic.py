# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Basic tests scenarios for snapshot save/restore."""

import logging
import platform
import tempfile
import pytest
from conftest import _test_images_s3_bucket
from framework.artifacts import ArtifactCollection, ArtifactSet
from framework.matrix import TestMatrix, TestContext
from framework.builder import MicrovmBuilder, SnapshotBuilder, SnapshotType
import host_tools.network as net_tools  # pylint: disable=import-error
import host_tools.drive as drive_tools


def _guest_run_fio_iteration(ssh_connection, iteration):
    fio = """fio --filename=/dev/vda --direct=1 --rw=randread --bs=4k \
        --ioengine=libaio --iodepth=16 --runtime=10 --numjobs=4 --time_based \
        --group_reporting --name=iops-test-job --eta-newline=1 --readonly"""
    ssh_cmd = "screen -L -Logfile /tmp/fio{} -dmS test{} {}"
    ssh_cmd = ssh_cmd.format(iteration, iteration, fio)
    exit_code, _, _ = ssh_connection.execute_command(ssh_cmd)
    assert exit_code == 0


def _get_guest_drive_size(ssh_connection, guest_dev_name='/dev/vdb'):
    # `lsblk` command outputs 2 lines to STDOUT:
    # "SIZE" and the size of the device, in bytes.
    blksize_cmd = "lsblk -b {} --output SIZE".format(guest_dev_name)
    _, stdout, stderr = ssh_connection.execute_command(blksize_cmd)
    assert stderr.read() == ''
    stdout.readline()  # skip "SIZE"
    return stdout.readline().strip()


def _test_seq_snapshots(context):
    logger = context.custom['logger']
    seq_len = context.custom['seq_len']
    vm_builder = context.custom['builder']
    snapshot_type = context.custom['snapshot_type']
    enable_diff_snapshots = snapshot_type == SnapshotType.DIFF

    logger.info("Testing {} with microvm: \"{}\", kernel {}, disk {} "
                .format(snapshot_type,
                        context.microvm.name(),
                        context.kernel.name(),
                        context.disk.name()))

    # Create a rw copy artifact.
    root_disk = context.disk.copy()
    # Get ssh key from read-only artifact.
    ssh_key = context.disk.ssh_key()
    # Create a fresh microvm from aftifacts.
    basevm = vm_builder.build(kernel=context.kernel,
                              disks=[root_disk],
                              ssh_key=ssh_key,
                              config=context.microvm,
                              enable_diff_snapshots=enable_diff_snapshots)

    network_config = net_tools.UniqueIPv4Generator.instance()
    _, host_ip, guest_ip = basevm.ssh_network_config(network_config,
                                                     '1',
                                                     tapname="tap0")
    logger.debug("Host IP: {}, Guest IP: {}".format(host_ip, guest_ip))

    # Add a scratch 128MB RW non-root block device.
    scratchdisk1 = drive_tools.FilesystemFile(tempfile.mktemp(), size=128)
    basevm.add_drive('scratch', scratchdisk1)

    # We will need netmask_len in build_from_snapshot() call later.
    netmask_len = network_config.get_netmask_len()
    basevm.start()
    ssh_connection = net_tools.SSHConnection(basevm.ssh_config)

    # Verify if guest can run commands.
    exit_code, _, _ = ssh_connection.execute_command("sync")
    assert exit_code == 0

    if 'check_patch_drive' in context.custom:
        # Update drive to have another backing file, double in size.
        new_file_size_mb = 2 * int(scratchdisk1.size()/(1024*1024))
        scratchdisk1 = drive_tools.FilesystemFile(tempfile.mktemp(),
                                                  new_file_size_mb)
        basevm.patch_drive('scratch', scratchdisk1)

    logger.info("Create {} #0.".format(snapshot_type))
    # Create a snapshot builder from a microvm.
    snapshot_builder = SnapshotBuilder(basevm)

    disks = [root_disk.local_path(), scratchdisk1.path]
    # Create base snapshot.
    snapshot = snapshot_builder.create(disks,
                                       ssh_key,
                                       snapshot_type)

    base_snapshot = snapshot
    basevm.kill()

    for i in range(seq_len):
        logger.info("Load snapshot #{}, mem {}".format(i, snapshot.mem))
        microvm, _ = vm_builder.build_from_snapshot(snapshot,
                                                    host_ip,
                                                    guest_ip,
                                                    netmask_len,
                                                    True,
                                                    enable_diff_snapshots)

        # Attempt to connect to resumed microvm.
        ssh_connection = net_tools.SSHConnection(microvm.ssh_config)

        # Start a new instance of fio on each iteration.
        _guest_run_fio_iteration(ssh_connection, i)

        if 'check_patch_drive' in context.custom:
            guest_drive_size = _get_guest_drive_size(ssh_connection)
            assert guest_drive_size == str(scratchdisk1.size())

        logger.info("Create snapshot #{}.".format(i + 1))

        # Create a snapshot builder from the currently running microvm.
        snapshot_builder = SnapshotBuilder(microvm)

        disks = [root_disk.local_path(), scratchdisk1.path]
        snapshot = snapshot_builder.create(disks,
                                           ssh_key,
                                           snapshot_type)

        # If we are testing incremental snapshots we must merge the base with
        # current layer.
        if snapshot_type == SnapshotType.DIFF:
            logger.info("Base: {}, Layer: {}".format(base_snapshot.mem,
                                                     snapshot.mem))
            snapshot.rebase_snapshot(base_snapshot)
            # Update the base for next iteration.
            base_snapshot = snapshot

        microvm.kill()


@pytest.mark.skipif(
    platform.machine() != "x86_64",
    reason="Not supported yet."
)
def test_patch_drive_snapshot(network_config,
                              bin_cloner_path):
    """Test scenario: 5 full sequential snapshots."""
    logger = logging.getLogger("snapshot_sequence")

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    # Testing matrix:
    # - Guest kernel: Linux 4.9/4.14
    # - Rootfs: Ubuntu 18.04
    # - Microvm: 2vCPU with 512 MB RAM
    # TODO: Multiple microvm sizes must be tested in the async pipeline.
    microvm_artifacts = ArtifactSet(artifacts.microvms(keyword="2vcpu_512mb"))
    kernel_artifacts = ArtifactSet(artifacts.kernels(keyword="4.14"))
    disk_artifacts = ArtifactSet(artifacts.disks(keyword="ubuntu"))

    # Create a test context and add builder, logger, network.
    test_context = TestContext()
    test_context.custom = {
        'builder': MicrovmBuilder(bin_cloner_path),
        'network_config': network_config,
        'logger': logger,
        'snapshot_type': SnapshotType.FULL,
        'seq_len': 1,
        'check_patch_drive': True
    }

    # Create the test matrix.
    test_matrix = TestMatrix(context=test_context,
                             artifact_sets=[
                                 microvm_artifacts,
                                 kernel_artifacts,
                                 disk_artifacts
                             ])

    test_matrix.run_test(_test_seq_snapshots)


@pytest.mark.skipif(
    platform.machine() != "x86_64",
    reason="Not supported yet."
)
def test_5_full_snapshots(network_config,
                          bin_cloner_path):
    """Test scenario: 5 full sequential snapshots."""
    logger = logging.getLogger("snapshot_sequence")

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    # Testing matrix:
    # - Guest kernel: Linux 4.9/4.14
    # - Rootfs: Ubuntu 18.04
    # - Microvm: 2vCPU with 512 MB RAM
    # TODO: Multiple microvm sizes must be tested in the async pipeline.
    microvm_artifacts = ArtifactSet(artifacts.microvms(keyword="2vcpu_512mb"))
    kernel_artifacts = ArtifactSet(artifacts.kernels())
    disk_artifacts = ArtifactSet(artifacts.disks(keyword="ubuntu"))

    # Create a test context and add builder, logger, network.
    test_context = TestContext()
    test_context.custom = {
        'builder': MicrovmBuilder(bin_cloner_path),
        'network_config': network_config,
        'logger': logger,
        'snapshot_type': SnapshotType.FULL,
        'seq_len': 5
    }

    # Create the test matrix.
    test_matrix = TestMatrix(context=test_context,
                             artifact_sets=[
                                 microvm_artifacts,
                                 kernel_artifacts,
                                 disk_artifacts
                             ])

    test_matrix.run_test(_test_seq_snapshots)


@pytest.mark.skipif(
    True,
    reason="Blocked by Github issue #1997"
)
def test_5_inc_snapshots(network_config,
                         bin_cloner_path):
    """Test scenario: 5 incremental snapshots with disk intensive workload."""
    logger = logging.getLogger("snapshot_sequence")

    artifacts = ArtifactCollection(_test_images_s3_bucket())
    # Testing matrix:
    # - Guest kernel: Linux 4.9/4.14
    # - Rootfs: Ubuntu 18.04
    # - Microvm: 2vCPU with 512 MB RAM
    # TODO: Multiple microvm sizes must be tested in the async pipeline.
    microvm_artifacts = ArtifactSet(artifacts.microvms(keyword="2vcpu_4096mb"))
    kernel_artifacts = ArtifactSet(artifacts.kernels())
    disk_artifacts = ArtifactSet(artifacts.disks(keyword="ubuntu"))

    # Create a test context and add builder, logger, network.
    test_context = TestContext()
    test_context.custom = {
        'builder': MicrovmBuilder(bin_cloner_path),
        'network_config': network_config,
        'logger': logger,
        'snapshot_type': SnapshotType.DIFF,
        'seq_len': 5
    }

    # Create the test matrix.
    test_matrix = TestMatrix(context=test_context,
                             artifact_sets=[
                                 microvm_artifacts,
                                 kernel_artifacts,
                                 disk_artifacts
                             ])

    test_matrix.run_test(_test_seq_snapshots)
