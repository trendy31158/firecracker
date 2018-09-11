"""Tests that the seccomp filters don't let blacklisted syscalls through."""

import os
import shutil
from subprocess import run

import pytest

import host_tools.cargo_build as host  # pylint:disable=import-error


@pytest.fixture
def tmp_jailer(test_session_root_path):
    """Build and returns the path to a binary using the `seccomp` crate."""

    jailer_srcdir = os.path.normpath(
        os.path.join(os.getcwd(), '../src/bin/demo_jailer/')
    )

    # The release binary path is created inside the testsession path as
    # `firecracker_binaries/release/x86_64-unknown-linux-musl/release/`
    release_binaries_path = os.path.join(
        host.CARGO_RELEASE_REL_PATH,
        host.RELEASE_BINARIES_REL_PATH
    )
    release_binaries_path = os.path.join(
        test_session_root_path,
        release_binaries_path
    )
    jailer_bin_path = os.path.normpath(
        os.path.join(
            release_binaries_path,
            'demo_jailer'
        )
    )

    os.makedirs(jailer_srcdir)
    with open(os.path.join(jailer_srcdir, 'main.rs'), 'w') as jailer_src:
        jailer_src.write("""
            extern crate seccomp;
            extern crate vmm;
            use std::env::args;
            use std::os::unix::process::CommandExt;
            use std::process::{Command, Stdio};
            fn main() {
                let args: Vec<String> = args().collect();
                let exec_file = &args[1];
                seccomp::setup_seccomp(seccomp::SeccompLevel::Basic(vmm::default_syscalls::ALLOWED_SYSCALLS)).unwrap();
                Command::new(exec_file).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).exec();
            }
            """)
    yield jailer_bin_path

    shutil.rmtree(jailer_srcdir)
    os.remove(jailer_bin_path)


def test_seccomp_ls(test_session_root_path, tmp_jailer):
    """Assert that the seccomp filters deny a blacklisted syscall."""
    # pylint: disable=redefined-outer-name
    # The fixture pattern causes a pylint false positive for that rule.

    # Path to the `ls` binary, which attempts to execute `SYS_access`,
    # blacklisted for Firecracker.
    ls_command_path = '/bin/ls'
    build_path = os.path.join(
        test_session_root_path,
        host.CARGO_RELEASE_REL_PATH
    )
    host.cargo_build(
        build_path,
        flags='--release --bin',
        extra_args='demo_jailer'
    )
    assert os.path.exists(tmp_jailer)

    # Compile the mini jailer.
    outcome = run([tmp_jailer, ls_command_path])

    # The seccomp filters should send SIGSYS (31) to the binary. `ls` doesn't
    # handle it, so it will exit with error.
    assert outcome.returncode != 0
