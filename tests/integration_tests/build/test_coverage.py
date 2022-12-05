# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests enforcing code coverage for production code."""

import os
import pytest

from framework import utils
from host_tools import proc

# We have different coverages based on the host kernel version. This is
# caused by io_uring, which is only supported by FC for kernels newer
# than 5.10.

# AMD has a slightly different coverage due to
# the appearance of the brand string. On Intel,
# this contains the frequency while on AMD it does not.
# Checkout the cpuid crate. In the future other
# differences may appear.
if utils.is_io_uring_supported():
    COVERAGE_DICT = {"Intel": 82.99, "AMD": 82.31, "ARM": 82.51}
else:
    COVERAGE_DICT = {"Intel": 80.15, "AMD": 79.48, "ARM": 79.59}

PROC_MODEL = proc.proc_type()

# Toolchain target architecture.
if ("Intel" in PROC_MODEL) or ("AMD" in PROC_MODEL):
    ARCH = "x86_64"
elif "ARM" in PROC_MODEL:
    ARCH = "aarch64"
else:
    raise Exception(f"Unsupported processor model ({PROC_MODEL})")

# Toolchain target.
# Currently profiling with `aarch64-unknown-linux-musl` is unsupported (see
# https://github.com/rust-lang/rustup/issues/3095#issuecomment-1280705619) therefore we profile and
# run coverage with the `gnu` toolchains and run unit tests with the `musl` toolchains.
TARGET = f"{ARCH}-unknown-linux-gnu"

# We allow coverage to have a max difference of `COVERAGE_MAX_DELTA` as percentage before failing
# the test.
COVERAGE_MAX_DELTA = 0.05

# grcov 0.8.* requires GLIBC >2.27, this is not present in ubuntu 18.04, when we update the docker
# container with a newer version of ubuntu we can also update this.
GRCOV_VERSION = "0.7.1"


@pytest.mark.timeout(400)
def test_coverage():
    """Test code coverage

    @type: build
    """
    # Get coverage target.
    processor_model = [item for item in COVERAGE_DICT if item in PROC_MODEL]
    assert len(processor_model) == 1, "Could not get processor model!"
    coverage_target = COVERAGE_DICT[processor_model[0]]

    # Re-direct to repository root.
    os.chdir("..")

    # Generate test profiles.
    utils.run_cmd(
        f'\
        env RUSTFLAGS="-Cinstrument-coverage" \
        LLVM_PROFILE_FILE="coverage-%p-%m.profraw" \
        cargo test --all --target={TARGET} -- --test-threads=1 \
    '
    )

    # Generate coverage report.
    utils.run_cmd(
        f'\
        cargo install --version {GRCOV_VERSION} grcov \
        && grcov . \
            -s . \
            --binary-path ./build/cargo_target/{TARGET}/debug/ \
            --excl-start "mod tests" \
            --ignore "build/*" \
            -t html \
            --branch \
            --ignore-not-existing \
            -o ./build/cargo_target/{TARGET}/debug/coverage \
    '
    )

    # Extract coverage from html report.
    #
    # The line looks like `<abbr title="44724 / 49237">90.83 %</abbr></p>` and is the first
    # occurrence of the `<abbr>` element in the file.
    #
    # When we update grcov to 0.8.* we can update this to pull the coverage from a generated .json
    # file.
    index = open(
        f"./build/cargo_target/{TARGET}/debug/coverage/index.html", encoding="utf-8"
    )
    index_contents = index.read()
    end = index_contents.find(" %</abbr></p>")
    start = index_contents[:end].rfind(">")
    coverage_str = index_contents[start + 1 : end]
    coverage = float(coverage_str)

    # Compare coverage.
    high = coverage_target * (1.0 + COVERAGE_MAX_DELTA)
    low = coverage_target * (1.0 - COVERAGE_MAX_DELTA)
    assert (
        coverage >= low
    ), f"Current code coverage ({coverage:.2f}%) is more than {COVERAGE_MAX_DELTA:.2f}% below \
            the target ({coverage_target:.2f}%)"
    assert (
        coverage <= high
    ), f"Current code coverage ({coverage:.2f}%) is more than {COVERAGE_MAX_DELTA:.2f}% above \
            the target ({coverage_target:.2f}%)"
