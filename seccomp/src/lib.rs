// Copyright 2018 Amazon.com, Inc. or its affiliates.  All Rights Reserved.

extern crate libc;

extern crate logger;
extern crate sys_util;

use self::logger::{Metric, METRICS};
use std::result::Result;

/// Integer values for the level of seccomp filtering used.
/// See `struct SeccompLevel` for more information about the different levels.
pub const SECCOMP_LEVEL_BASIC: u32 = 1;
pub const SECCOMP_LEVEL_NONE: u32 = 0;

/// BPF Instruction classes.
/// See /usr/include/linux/bpf_common.h .
const BPF_LD: u16 = 0x00;
const BPF_ALU: u16 = 0x04;
const BPF_JMP: u16 = 0x05;
const BPF_RET: u16 = 0x06;

/// BPF ld/ldx fields
/// See /usr/include/linux/bpf_common.h .
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;

/// BPF alu fields.
/// See /usr/include/linux/bpf_common.h .
const BPF_AND: u16 = 0x50;

/// BPF jmp fields.
/// See /usr/include/linux/bpf_common.h .
const BPF_JA: u16 = 0x00;
const BPF_JEQ: u16 = 0x10;
const BPF_JGT: u16 = 0x20;
const BPF_JGE: u16 = 0x30;
const BPF_K: u16 = 0x00;

/// Return codes for BPF programs.
///  See /usr/include/linux/seccomp.h .
const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;
const SECCOMP_RET_KILL: u32 = 0x00000000;
const SECCOMP_RET_TRAP: u32 = 0x00030000;

/// x86_64 architecture identifier.
/// See /usr/include/linux/audit.h .
/// Defined as:
/// `#define AUDIT_ARCH_X86_64	(EM_X86_64|__AUDIT_ARCH_64BIT|__AUDIT_ARCH_LE)`
const AUDIT_ARCH_X86_64: u32 = 62 | 0x80000000 | 0x40000000;

/// The offset of `si_syscall` (offending syscall identifier) within the siginfo structure
/// expressed as an `(u)int*`.
/// Offset `6` for an `i32` field means that the needed information is located at `6 * sizeof(i32)`.
/// See /usr/include/linux/signal.h for the C struct definition.
/// See https://github.com/rust-lang/libc/issues/716 for why the offset is different in Rust.
const SI_OFF_SYSCALL: isize = 6;

/// The maximum number of a syscall argument.
/// A syscall can have at most 6 arguments.
/// Arguments are numbered from 0 to 5.
const ARG_NUMBER_MAX: u8 = 5;

/// The maximum number of BPF statements that a condition will be translated into.
const CONDITION_MAX_LEN: u16 = 6;

/// `struct seccomp_data` offsets and sizes of fields in bytes:
///
/// ```c
/// struct seccomp_data {
///     int nr;
///     __u32 arch;
///     __u64 instruction_pointer;
///     __u64 args[6];
/// };
/// ```
const SECCOMP_DATA_ARGS_OFFSET: u8 = 16;
const SECCOMP_DATA_ARG_SIZE: u8 = 8;

/// Specifies the type of seccomp filtering used.
pub enum SeccompLevel<'a> {
    /// Seccomp filtering by analysing syscall number.
    Basic(&'a [i64]),
    /// No seccomp filtering.
    None,
}

/// Seccomp errors.
pub enum Error {
    /// Argument number that exceeds the maximum value.
    InvalidArgumentNumber,
}

/// Comparison to perform when matching a condition.
#[derive(PartialEq)]
pub enum SeccompCmpOp {
    /// Argument value is equal to the specified value.
    Eq,
    /// Argument value is greater than or equal to the specified value.
    Ge,
    /// Argument value is greater than specified value.
    Gt,
    /// Argument value is less than or equal to the specified value.
    Le,
    /// Argument value is less than specified value.
    Lt,
    /// Masked bits of argument value are equal to masked bits of specified value.
    MaskedEq(u64),
    /// Argument value is not equal to specified value.
    Ne,
}

/// Condition that syscall must match in order to satisfy a rule.
pub struct SeccompCondition {
    /// Number of the argument value that is to be compared.
    arg_number: u8,
    /// Comparison to perform.
    operator: SeccompCmpOp,
    /// The value that will be compared with the argument value.
    value: u64,
}

/// BPF instruction structure definition.
/// See /usr/include/linux/filter.h .
#[repr(C)]
#[derive(Debug, PartialEq)]
struct sock_filter {
    pub code: ::std::os::raw::c_ushort,
    pub jt: ::std::os::raw::c_uchar,
    pub jf: ::std::os::raw::c_uchar,
    pub k: ::std::os::raw::c_uint,
}

/// BPF structure definition for filter array.
/// See /usr/include/linux/filter.h .
#[repr(C)]
#[derive(Debug)]
struct sock_fprog {
    pub len: ::std::os::raw::c_ushort,
    pub filter: *const sock_filter,
}

impl SeccompCondition {
    /// Creates a new `SeccompCondition`.
    pub fn new(arg_number: u8, operator: SeccompCmpOp, value: u64) -> Result<Self, Error> {
        // Checks that the given argument number is valid.
        if arg_number > ARG_NUMBER_MAX {
            return Err(Error::InvalidArgumentNumber);
        }

        Ok(Self {
            arg_number,
            operator,
            value,
        })
    }

    /// Helper method.
    /// Returns most significant half, least significant half of the `value` field of
    /// `SeccompCondition`, also returns the offsets of the most significant and least significant
    /// half of the argument specified by `arg_number` relative to `struct seccomp_data` passed to
    /// the BPF program by the kernel.
    fn value_segments(&self) -> (u32, u32, u8, u8) {
        // Splits the specified value into its most significant and least significant halves.
        let (msb, lsb) = ((self.value >> 32) as u32, self.value as u32);

        // Offset to the argument specified by `arg_number`.
        let arg_offset = SECCOMP_DATA_ARGS_OFFSET + self.arg_number * SECCOMP_DATA_ARG_SIZE;

        // Extracts offsets of most significant and least significant halves of argument.
        let (msb_offset, lsb_offset) = {
            #[cfg(target_endian = "big")]
            {
                (arg_offset, arg_offset + SECCOMP_DATA_ARG_SIZE / 2)
            }
            #[cfg(target_endian = "little")]
            {
                (arg_offset + SECCOMP_DATA_ARG_SIZE / 2, arg_offset)
            }
        };

        (msb, lsb, msb_offset, lsb_offset)
    }

    /// Helper methods, translating conditions into BPF statements, based on the operator of the
    /// condition.
    ///
    /// The `offset` parameter is a given jump offset to the start of the next rule. The jump is
    /// performed if the condition fails and thus the current rule does not match so `seccomp` tries
    /// to match the next rule by jumping out of the current rule.
    ///
    /// In case the condition is part of the last rule, the jump offset is to the default action of
    /// respective context.
    ///
    /// The most significant and least significant halves of the argument value are compared
    /// separately since the BPF operand and accumulator are 4 bytes whereas an argument value is 8.
    fn into_eq_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, offset + 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, lsb, 0, offset),
        ]
    }

    fn into_ge_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, msb, 3, 0),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, offset + 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGE + BPF_K, lsb, 0, offset),
        ]
    }

    fn into_gt_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, msb, 3, 0),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, offset + 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, lsb, 0, offset),
        ]
    }

    fn into_le_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, msb, offset + 3, 0),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, lsb, offset, 0),
        ]
    }

    fn into_lt_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGT + BPF_K, msb, offset + 3, 0),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JGE + BPF_K, lsb, offset, 0),
        ]
    }

    fn into_masked_eq_bpf(self, offset: u8, mask: u64) -> Vec<sock_filter> {
        let (_, _, msb_offset, lsb_offset) = self.value_segments();
        let masked_value = self.value & mask;
        let (msb, lsb) = ((masked_value >> 32) as u32, masked_value as u32);
        let (mask_msb, mask_lsb) = ((mask >> 32) as u32, mask as u32);

        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_STMT(BPF_ALU + BPF_AND + BPF_K, mask_msb),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, offset + 3),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_STMT(BPF_ALU + BPF_AND + BPF_K, mask_lsb),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, lsb, 0, offset),
        ]
    }

    fn into_ne_bpf(self, offset: u8) -> Vec<sock_filter> {
        let (msb, lsb, msb_offset, lsb_offset) = self.value_segments();
        vec![
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, msb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, msb, 0, 2),
            BPF_STMT(BPF_LD + BPF_W + BPF_ABS, lsb_offset as u32),
            BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, lsb, offset, 0),
        ]
    }

    /// Translates `SeccompCondition` into BPF statements.
    fn into_bpf(self, offset: u8) -> Vec<sock_filter> {
        let result = match self.operator {
            SeccompCmpOp::Eq => self.into_eq_bpf(offset),
            SeccompCmpOp::Ge => self.into_ge_bpf(offset),
            SeccompCmpOp::Gt => self.into_gt_bpf(offset),
            SeccompCmpOp::Le => self.into_le_bpf(offset),
            SeccompCmpOp::Lt => self.into_lt_bpf(offset),
            SeccompCmpOp::MaskedEq(mask) => self.into_masked_eq_bpf(offset, mask),
            SeccompCmpOp::Ne => self.into_ne_bpf(offset),
        };

        // Regression testing that the `CONDITION_MAX_LEN` constant was properly updated.
        assert!(result.len() <= CONDITION_MAX_LEN as usize);

        result
    }
}

/// Builds the array of filter instructions and sends them to the kernel.
pub fn setup_seccomp(level: SeccompLevel) -> Result<(), i32> {
    let mut filters = Vec::new();

    filters.extend(VALIDATE_ARCHITECTURE());

    // Load filters according to specified filter level.
    match level {
        SeccompLevel::Basic(allowed_syscalls) => {
            filters.extend(EXAMINE_SYSCALL());
            for &syscall in allowed_syscalls {
                filters.extend(ALLOW_SYSCALL(syscall));
            }
            filters.extend(SIGNAL_PROCESS());
        }
        SeccompLevel::None => {
            return Ok(());
        }
    }

    unsafe {
        {
            let rc = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            if rc != 0 {
                return Err(*libc::__errno_location());
            }
        }

        let filter = sock_fprog {
            len: filters.len() as u16,
            filter: filters.as_ptr(),
        };
        let filter_ptr = &filter as *const sock_fprog;

        {
            let rc = libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, filter_ptr);
            if rc != 0 {
                return Err(*libc::__errno_location());
            }
        }
    }

    Ok(())
}

pub fn setup_sigsys_handler() -> Result<(), sys_util::Error> {
    return unsafe {
        sys_util::register_signal_handler(
            libc::SIGSYS,
            sys_util::SignalHandler::Siginfo(sigsys_handler),
            false,
        )
    };
}

extern "C" fn sigsys_handler(
    num: libc::c_int,
    info: *mut libc::siginfo_t,
    _unused: *mut libc::c_void,
) {
    if num != libc::SIGSYS {
        return;
    }
    let syscall = unsafe { *(info as *const i32).offset(SI_OFF_SYSCALL) as usize };
    METRICS.seccomp.num_faults.inc();
    METRICS.seccomp.bad_syscalls[syscall].inc();
}

/// Builds a "jump" BPF instruction.
#[allow(non_snake_case)]
fn BPF_JUMP(code: u16, k: u32, jt: u8, jf: u8) -> sock_filter {
    sock_filter { code, jt, jf, k }
}

/// Builds a "statement" BPF instruction.
#[allow(non_snake_case)]
fn BPF_STMT(code: u16, k: u32) -> sock_filter {
    sock_filter {
        code,
        jt: 0,
        jf: 0,
        k,
    }
}

/// Builds a sequence of BPF instructions that validate the underlying architecture.
#[allow(non_snake_case)]
fn VALIDATE_ARCHITECTURE() -> Vec<sock_filter> {
    vec![
        BPF_STMT(BPF_LD + BPF_W + BPF_ABS, 4),
        BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, AUDIT_ARCH_X86_64, 1, 0),
        BPF_STMT(BPF_RET + BPF_K, SECCOMP_RET_KILL),
    ]
}

/// Builds a sequence of BPF instructions that are followed by syscall examination.
#[allow(non_snake_case)]
fn EXAMINE_SYSCALL() -> Vec<sock_filter> {
    vec![BPF_STMT(BPF_LD + BPF_W + BPF_ABS, 0)]
}

/// Builds a sequence of BPF instructions that allow a syscall to go through.
#[allow(non_snake_case)]
fn ALLOW_SYSCALL(syscall_number: i64) -> Vec<sock_filter> {
    vec![
        BPF_JUMP(BPF_JMP + BPF_JEQ + BPF_K, syscall_number as u32, 0, 1),
        BPF_STMT(BPF_RET + BPF_K, SECCOMP_RET_ALLOW),
    ]
}

/// Builds a sequence of BPF instructions that emit SIGSYS when a syscall is denied.
#[allow(non_snake_case)]
fn SIGNAL_PROCESS() -> Vec<sock_filter> {
    vec![BPF_STMT(BPF_RET + BPF_K, SECCOMP_RET_TRAP)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_signal_handler() {
        assert!(setup_sigsys_handler().is_ok());

        // Syscalls that have to be allowed in order for the test to work.
        const REQUIRED_SYSCALLS: &[i64] = &[
            libc::SYS_exit,
            libc::SYS_futex,
            libc::SYS_munmap,
            libc::SYS_rt_sigprocmask,
            libc::SYS_rt_sigreturn,
            libc::SYS_set_tid_address,
            libc::SYS_sigaltstack,
        ];
        assert!(setup_seccomp(SeccompLevel::Basic(REQUIRED_SYSCALLS)).is_ok());

        // Calls the blacklisted SYS_getpid.
        let _pid = process::id();

        // The signal handler should let the program continue.
        assert!(true);

        // The reason this test doesn't check the failure metrics as well is that the signal handler
        // doesn't work right with kcov - possibly because the process is being pinned to 1 core.
    }

    #[test]
    fn test_bpf_functions() {
        {
            let ret = VALIDATE_ARCHITECTURE();
            let instructions = vec![
                sock_filter {
                    code: 32,
                    jt: 0,
                    jf: 0,
                    k: 4,
                },
                sock_filter {
                    code: 21,
                    jt: 1,
                    jf: 0,
                    k: 0xC000003E,
                },
                sock_filter {
                    code: 6,
                    jt: 0,
                    jf: 0,
                    k: 0,
                },
            ];
            assert_eq!(ret, instructions);
        }

        {
            let ret = EXAMINE_SYSCALL();
            let instructions = vec![sock_filter {
                code: 32,
                jt: 0,
                jf: 0,
                k: 0,
            }];
            assert_eq!(ret, instructions);
        }

        {
            let ret = ALLOW_SYSCALL(123);
            let instructions = vec![
                sock_filter {
                    code: 21,
                    jt: 0,
                    jf: 1,
                    k: 123,
                },
                sock_filter {
                    code: 6,
                    jt: 0,
                    jf: 0,
                    k: 0x7FFF0000,
                },
            ];
            assert_eq!(ret, instructions);
        }

        {
            let ret = SIGNAL_PROCESS();
            let instructions = vec![sock_filter {
                code: 6,
                jt: 0,
                jf: 0,
                k: 0x30000,
            }];
            assert_eq!(ret, instructions);
        }
    }
}
