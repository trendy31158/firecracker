# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests pertaining to line/branch test coverage for the Firecracker code base.

# TODO

- Put the coverage in `s3://spec.firecracker` and update it automatically.
  target should be put in `s3://spec.firecracker` and automatically updated.
"""


import os
import platform
import re
import shutil
import pytest

import framework.utils as utils
import host_tools.cargo_build as host  # pylint: disable=import-error
import host_tools.proc as proc

# AMD has a slightly different coverage due to
# the appearance of the brand string. On Intel,
# this contains the frequency while on AMD it does not.
# Checkout the cpuid crate. In the future other
# differences may appear.
COVERAGE_DICT = {"Intel": 84.82, "AMD": 84.22, "ARM": 83.14}

PROC_MODEL = proc.proc_type()

COVERAGE_MAX_DELTA = 0.05

CARGO_KCOV_REL_PATH = os.path.join(host.CARGO_BUILD_REL_PATH, 'kcov')

KCOV_COVERAGE_FILE = 'index.js'
"""kcov will aggregate coverage data in this file."""

KCOV_COVERED_LINES_REGEX = r'"covered_lines":"(\d+)"'
"""Regex for extracting number of total covered lines found by kcov."""

KCOV_TOTAL_LINES_REGEX = r'"total_lines" : "(\d+)"'
"""Regex for extracting number of total executable lines found by kcov."""

SECCOMPILER_BUILD_DIR = '../build/seccompiler'


@pytest.mark.timeout(400)
def test_coverage(test_fc_session_root_path, test_session_tmp_path):
    """Test line coverage with kcov.

    The result is extracted from the $KCOV_COVERAGE_FILE file created by kcov
    after a coverage run.
    """
    proc_model = [item for item in COVERAGE_DICT if item in PROC_MODEL]
    assert len(proc_model) == 1, "Could not get processor model!"
    coverage_target_pct = COVERAGE_DICT[proc_model[0]]
    exclude_pattern = (
        '${CARGO_HOME:-$HOME/.cargo/},'
        'build/,'
        'tests/,'
        'usr/lib/gcc,'
        'lib/x86_64-linux-gnu/,'
        'test_utils.rs,'
        # The following files/directories are auto-generated
        'bootparam.rs,'
        'elf.rs,'
        'mpspec.rs,'
        'msr_index.rs,'
        '_gen'
    )
    exclude_region = '\'mod tests {\''
    target = "{}-unknown-linux-musl".format(platform.machine())

    cmd = (
        'RUSTFLAGS="{}" CARGO_TARGET_DIR={} cargo kcov --all '
        '--target {} --output {} -- '
        '--exclude-pattern={} '
        '--exclude-region={} --verify'
    ).format(
        host.get_rustflags(),
        os.path.join(test_fc_session_root_path, CARGO_KCOV_REL_PATH),
        target,
        test_session_tmp_path,
        exclude_pattern,
        exclude_region
    )
    # We remove the seccompiler custom build directory, created by the
    # vmm-level `build.rs`.
    # If we don't delete it before and after running the kcov command, we will
    # run into linker errors.
    shutil.rmtree(SECCOMPILER_BUILD_DIR, ignore_errors=True)
    # By default, `cargo kcov` passes `--exclude-pattern=$CARGO_HOME --verify`
    # to kcov. To pass others arguments, we need to include the defaults.
    utils.run_cmd(cmd)

    shutil.rmtree(SECCOMPILER_BUILD_DIR)

    coverage_file = os.path.join(test_session_tmp_path, KCOV_COVERAGE_FILE)
    with open(coverage_file) as cov_output:
        contents = cov_output.read()
        covered_lines = int(re.findall(KCOV_COVERED_LINES_REGEX, contents)[0])
        total_lines = int(re.findall(KCOV_TOTAL_LINES_REGEX, contents)[0])
        coverage = covered_lines / total_lines * 100
    print("Number of executable lines: {}".format(total_lines))
    print("Number of covered lines: {}".format(covered_lines))
    print("Thus, coverage is: {:.2f}%".format(coverage))

    coverage_low_msg = (
        'Current code coverage ({:.2f}%) is below the target ({}%).'
        .format(coverage, coverage_target_pct)
    )

    min_coverage = coverage_target_pct - COVERAGE_MAX_DELTA
    assert coverage >= min_coverage, coverage_low_msg

    # Get the name of the variable that needs updating.
    namespace = globals()
    cov_target_name = [name for name in namespace if namespace[name]
                       is COVERAGE_DICT][0]

    coverage_high_msg = (
        'Current code coverage ({:.2f}%) is above the target ({}%).\n'
        'Please update the value of {}.'
        .format(coverage, coverage_target_pct, cov_target_name)
    )

    assert coverage - coverage_target_pct <= COVERAGE_MAX_DELTA,\
        coverage_high_msg
