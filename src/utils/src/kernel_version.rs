// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io::Error as IoError;
use std::result::Result;

use libc::{uname, utsname};

#[derive(Debug)]
pub enum Error {
    Uname(IoError),
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidFormat,
    InvalidInt(std::num::ParseIntError),
}

#[derive(PartialEq, PartialOrd)]
pub struct KernelVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl KernelVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn get() -> Result<Self, Error> {
        let mut name: utsname = utsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };
        let res = unsafe { uname((&mut name) as *mut utsname) };

        if res < 0 {
            return Err(Error::Uname(IoError::last_os_error()));
        }

        let (major, minor, patch) = Self::parse(
            String::from_utf8(name.release.iter().map(|c| *c as u8).collect())
                .map_err(Error::InvalidUtf8)?,
        )?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    fn parse(release: String) -> Result<(u16, u16, u16), Error> {
        let tokens: Vec<_> = release.split('.').collect();

        if tokens.len() < 3 {
            return Err(Error::InvalidFormat);
        }

        // Parse the `patch`, since it may contain other tokens as well.
        let mut patch = tokens[2];
        if let Some(index) = patch.find(|c: char| !c.is_digit(10)) {
            patch = &patch[..index];
        }

        Ok((
            tokens[0].parse().map_err(Error::InvalidInt)?,
            tokens[1].parse().map_err(Error::InvalidInt)?,
            patch.parse().map_err(Error::InvalidInt)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get() {
        assert!(KernelVersion::get().is_ok());
    }

    #[test]
    fn test_parse_valid() {
        assert_eq!(
            KernelVersion::parse("5.10.0".to_string()).unwrap(),
            (5, 10, 0)
        );
        assert_eq!(
            KernelVersion::parse("5.10.50".to_string()).unwrap(),
            (5, 10, 50)
        );
        KernelVersion::parse("5.10.50-38.132.amzn2int.x86_64".to_string()).expect("see here");
        assert_eq!(
            KernelVersion::parse("5.10.50-38.132.amzn2int.x86_64".to_string()).unwrap(),
            (5, 10, 50)
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert!(KernelVersion::parse("".to_string()).is_err());
        assert!(KernelVersion::parse("ffff".to_string()).is_err());
        assert!(KernelVersion::parse("ffff.55.0".to_string()).is_err());
        assert!(KernelVersion::parse("5.10.".to_string()).is_err());
        assert!(KernelVersion::parse("5.0".to_string()).is_err());
        assert!(KernelVersion::parse("5.0fff".to_string()).is_err());
    }

    #[test]
    fn test_cmp() {
        // Comparing major.
        assert!(KernelVersion::new(4, 0, 0) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(4, 0, 0) > KernelVersion::new(3, 10, 15));

        // Comparing minor.
        assert!(KernelVersion::new(5, 0, 20) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 20, 20) > KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 100, 20) > KernelVersion::new(5, 20, 0));

        // Comparing patch.
        assert!(KernelVersion::new(5, 0, 20) < KernelVersion::new(5, 10, 15));
        assert!(KernelVersion::new(5, 0, 20) > KernelVersion::new(4, 10, 15));

        // Equal.
        assert!(KernelVersion::new(5, 0, 20) == KernelVersion::new(5, 0, 20));
        assert!(KernelVersion::new(5, 0, 20) >= KernelVersion::new(5, 0, 20));
        assert!(KernelVersion::new(5, 0, 20) <= KernelVersion::new(5, 0, 20));
    }
}
