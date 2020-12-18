# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Helper functions for testing CPU identification functionality."""

import subprocess
from enum import Enum, auto

from framework.utils import run_cmd
import host_tools.network as net_tools


class CpuVendor(Enum):
    """CPU vendors enum."""

    AMD = auto()
    INTEL = auto()


def get_cpu_vendor():
    """Return the CPU vendor."""
    brand_str = subprocess.check_output("lscpu", shell=True).strip().decode()
    if 'AuthenticAMD' in brand_str:
        return CpuVendor.AMD
    return CpuVendor.INTEL


def get_cpu_model_name():
    """Return the CPU model name."""
    _, stdout, _ = run_cmd("cat /proc/cpuinfo | grep 'model name' | uniq")
    info = stdout.strip().split(sep=":")
    assert len(info) == 2
    return info[1].strip()


def check_guest_cpuid_output(vm, guest_cmd, expected_header,
                             expected_separator,
                             expected_key_value_store):
    """Parse cpuid output inside guest and match with expected one."""
    ssh_connection = net_tools.SSHConnection(vm.ssh_config)
    _, stdout, stderr = ssh_connection.execute_command(guest_cmd)

    assert stderr.read() == ''
    while True:
        line = stdout.readline()
        if line != '':
            # All the keys have been matched. Stop.
            if not expected_key_value_store:
                break

            # Try to match the header if needed.
            if expected_header not in (None, ''):
                if line.strip() == expected_header:
                    expected_header = None
                continue

            # See if any key matches.
            # We Use a try-catch block here since line.split() may fail.
            try:
                [key, value] = list(
                    map(lambda x: x.strip(), line.split(expected_separator)))
            except ValueError:
                continue

            if key in expected_key_value_store.keys():
                assert value == expected_key_value_store[key], \
                    "%s does not have the expected value" % key
                del expected_key_value_store[key]

        else:
            break

    assert not expected_key_value_store, \
        "some keys in dictionary have not been found in the output: %s" \
        % expected_key_value_store


def read_guest_file(vm, file):
    """Parse cpuid output inside guest and match with expected one."""
    ssh_connection = net_tools.SSHConnection(vm.ssh_config)
    _, stdout, stderr = ssh_connection.execute_command("cat {}".format(file))
    assert stderr.read() == ""
    return stdout.read().strip()
