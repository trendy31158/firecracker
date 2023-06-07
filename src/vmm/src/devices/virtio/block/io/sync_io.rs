// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::result::Result;

use utils::vm_memory::{
    GuestAddress, GuestMemory, GuestMemoryError, GuestMemoryMmap, ReadVolatile, WriteVolatile,
};

#[derive(Debug)]
pub enum Error {
    Flush(std::io::Error),
    Seek(std::io::Error),
    SyncAll(std::io::Error),
    Transfer(GuestMemoryError),
}

#[derive(Debug)]
pub struct SyncFileEngine {
    file: File,
}

// SAFETY: `File` is send and ultimately a POD.
unsafe impl Send for SyncFileEngine {}

impl SyncFileEngine {
    #[tracing::instrument(level = "trace", ret)]
    pub fn from_file(file: File) -> SyncFileEngine {
        SyncFileEngine { file }
    }

    #[cfg(test)]
    #[tracing::instrument(level = "trace", ret)]
    pub fn file(&self) -> &File {
        &self.file
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn read(
        &mut self,
        offset: u64,
        mem: &GuestMemoryMmap,
        addr: GuestAddress,
        count: u32,
    ) -> Result<u32, Error> {
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(Error::Seek)?;
        mem.get_slice(addr, count as usize)
            .and_then(|mut slice| Ok(self.file.read_exact_volatile(&mut slice)?))
            .map_err(Error::Transfer)?;
        Ok(count)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn write(
        &mut self,
        offset: u64,
        mem: &GuestMemoryMmap,
        addr: GuestAddress,
        count: u32,
    ) -> Result<u32, Error> {
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(Error::Seek)?;
        mem.get_slice(addr, count as usize)
            .and_then(|slice| Ok(self.file.write_all_volatile(&slice)?))
            .map_err(Error::Transfer)?;
        Ok(count)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn flush(&mut self) -> Result<(), Error> {
        // flush() first to force any cached data out of rust buffers.
        self.file.flush().map_err(Error::Flush)?;
        // Sync data out to physical media on host.
        self.file.sync_all().map_err(Error::SyncAll)
    }
}
