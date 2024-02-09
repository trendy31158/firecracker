# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Test that the process startup time up to socket bind is within spec."""

import os
import time

import pytest

from framework.properties import global_props
from host_tools.cargo_build import run_seccompiler_bin


@pytest.fixture
def startup_time(metrics, record_property):
    """Fixture to capture the startup time"""
    metrics.set_dimensions(
        {
            "instance": global_props.instance,
            "cpu_model": global_props.cpu_model,
            "host_kernel": "linux-" + global_props.host_linux_version,
        }
    )

    def record_startup_time(startup_time):
        metrics.put_metric("startup_time", startup_time, unit="Microseconds")
        record_property("startup_time_μs", startup_time)

    return record_startup_time


def test_startup_time_new_pid_ns(uvm_plain, startup_time):
    """
    Check startup time when jailer is spawned in a new PID namespace.
    """
    microvm = uvm_plain
    microvm.jailer.new_pid_ns = True
    startup_time(_test_startup_time(microvm))


def test_startup_time_daemonize(uvm_plain, startup_time):
    """
    Check startup time when jailer detaches Firecracker from the controlling terminal.
    """
    microvm = uvm_plain
    startup_time(_test_startup_time(microvm))


def test_startup_time_custom_seccomp(uvm_plain, startup_time):
    """
    Check the startup time when using custom seccomp filters.
    """
    microvm = uvm_plain
    _custom_filter_setup(microvm)
    startup_time(_test_startup_time(microvm))


def _test_startup_time(microvm):
    microvm.spawn()
    microvm.basic_config(vcpu_count=2, mem_size_mib=1024)
    test_start_time = time.time()
    microvm.start()
    time.sleep(0.4)

    # The metrics should be at index 1.
    # Since metrics are flushed at InstanceStart, the first line will suffice.
    datapoints = microvm.get_all_metrics()
    test_end_time = time.time()
    metrics = datapoints[0]
    startup_time_us = metrics["api_server"]["process_startup_time_us"]
    cpu_startup_time_us = metrics["api_server"]["process_startup_time_cpu_us"]

    print(
        "Process startup time is: {} us ({} CPU us)".format(
            startup_time_us, cpu_startup_time_us
        )
    )

    assert cpu_startup_time_us > 0
    # Check that startup time is not a huge value
    # This is to catch issues like the ones introduced in PR
    # https://github.com/firecracker-microvm/firecracker/pull/4305
    test_time_delta_us = (test_end_time - test_start_time) * 1000 * 1000
    assert startup_time_us < test_time_delta_us
    assert cpu_startup_time_us < test_time_delta_us
    return cpu_startup_time_us


def _custom_filter_setup(test_microvm):
    bpf_path = os.path.join(test_microvm.path, "bpf.out")

    run_seccompiler_bin(bpf_path)

    test_microvm.create_jailed_resource(bpf_path)
    test_microvm.jailer.extra_args.update({"seccomp-filter": "bpf.out"})
