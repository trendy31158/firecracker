// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Provides functionality for a userspace page fault handler
//! which loads the whole region from the backing memory file
//! when a page fault occurs.

use std::collections::HashMap;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::{mem, ptr};

use libc::c_void;
use nix::poll::{poll, PollFd, PollFlags};
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use serde::Deserialize;
use userfaultfd::Uffd;
use utils::get_page_size;

use utils::sock_ctrl_msg::ScmSocket;

// This is the same with the one used in src/vmm.
/// This describes the mapping between Firecracker base virtual address and offset in the
/// buffer or file backend for a guest memory region. It is used to tell an external
/// process/thread where to populate the guest memory data for this range.
///
/// E.g. Guest memory contents for a region of `size` bytes can be found in the backend
/// at `offset` bytes from the beginning, and should be copied/populated into `base_host_address`.
#[derive(Clone, Debug, Deserialize)]
pub struct GuestRegionUffdMapping {
    /// Base host virtual address where the guest memory contents for this region
    /// should be copied/populated.
    pub base_host_virt_addr: u64,
    /// Region size.
    pub size: usize,
    /// Offset in the backend file/buffer where the region contents are.
    pub offset: u64,
}

struct MemRegion {
    mapping: GuestRegionUffdMapping,
    page_states: HashMap<u64, MemPageState>,
}

struct UffdPfHandler {
    mem_regions: Vec<MemRegion>,
    backing_buffer: *const u8,
    uffd: Uffd,
    // Not currently used but included to demonstrate how a page fault handler can
    // fetch Firecracker's PID in order to make it aware of any crashes/exits.
    _firecracker_pid: u32,
}

#[derive(Clone)]
enum MemPageState {
    Uninitialized,
    FromFile,
    Removed,
    Anonymous,
}

impl UffdPfHandler {
    pub fn from_unix_stream(stream: UnixStream, data: *const u8, size: usize) -> Self {
        let mut message_buf = vec![0u8; 1024];
        let (bytes_read, file) = stream
            .recv_with_fd(&mut message_buf[..])
            .expect("Cannot recv_with_fd");
        message_buf.resize(bytes_read, 0);

        let body = String::from_utf8(message_buf).unwrap();
        let file = file.expect("Uffd not passed through UDS!");

        let mappings = serde_json::from_str::<Vec<GuestRegionUffdMapping>>(&body)
            .expect("Cannot deserialize memory mappings.");
        let memsize: usize = mappings.iter().map(|r| r.size).sum();

        // Make sure memory size matches backing data size.
        assert_eq!(memsize, size);

        let uffd = unsafe { Uffd::from_raw_fd(file.into_raw_fd()) };

        let creds: libc::ucred = get_peer_process_credentials(stream);

        let mem_regions = create_mem_regions(&mappings);

        Self {
            mem_regions,
            backing_buffer: data,
            uffd,
            _firecracker_pid: creds.pid as u32,
        }
    }

    fn update_mem_state_mappings(&mut self, start: u64, end: u64, state: &MemPageState) {
        for region in self.mem_regions.iter_mut() {
            for (key, value) in region.page_states.iter_mut() {
                if key >= &start && key < &end {
                    *value = state.clone();
                }
            }
        }
    }

    fn populate_from_file(&self, region: &MemRegion) -> (u64, u64) {
        let src = self.backing_buffer as u64 + region.mapping.offset;
        let start_addr = region.mapping.base_host_virt_addr;
        let len = region.mapping.size;
        // Populate whole region from backing mem-file.
        // This offers an example of how memory can be loaded in RAM,
        // however this can be adjusted to accommodate use case needs.
        let ret = unsafe {
            self.uffd
                .copy(src as *const _, start_addr as *mut _, len, true)
                .expect("Uffd copy failed")
        };

        // Make sure the UFFD copied some bytes.
        assert!(ret > 0);

        return (start_addr, start_addr + len as u64);
    }

    fn zero_out(&mut self, addr: u64) -> (u64, u64) {
        let page_size = get_page_size().unwrap();

        let ret = unsafe {
            self.uffd
                .zeropage(addr as *mut _, page_size, true)
                .expect("Uffd zeropage failed")
        };
        // Make sure the UFFD zeroed out some bytes.
        assert!(ret > 0);

        return (addr, addr + page_size as u64);
    }

    fn serve_pf(&mut self, addr: *mut u8) {
        let page_size = get_page_size().unwrap();

        // Find the start of the page that the current faulting address belongs to.
        let dst = (addr as usize & !(page_size as usize - 1)) as *mut c_void;
        let fault_page_addr = dst as u64;

        // Get the state of the current faulting page.
        for region in self.mem_regions.iter() {
            match region.page_states.get(&fault_page_addr) {
                // Our simple PF handler has a simple strategy:
                // There exist 4 states in which a memory page can be in:
                // 1. Uninitialized - page was never touched
                // 2. FromFile - the page is populated with content from snapshotted memory file
                // 3. Removed - MADV_DONTNEED was called due to balloon inflation
                // 4. Anonymous - page was zeroed out -> this implies that more than one page fault
                //    event was received. This can be a consequence of guest reclaiming back its
                //    memory from the host (through balloon device)
                Some(MemPageState::Uninitialized) | Some(MemPageState::FromFile) => {
                    let (start, end) = self.populate_from_file(region);
                    self.update_mem_state_mappings(start, end, &MemPageState::FromFile);
                    return;
                }
                Some(MemPageState::Removed) | Some(MemPageState::Anonymous) => {
                    let (start, end) = self.zero_out(fault_page_addr);
                    self.update_mem_state_mappings(start, end, &MemPageState::Anonymous);
                    return;
                }
                None => {
                    ();
                }
            }
        }

        panic!(
            "Could not find addr: {:?} within guest region mappings.",
            addr
        );
    }

    fn run_loop(&mut self) {
        let pollfd = PollFd::new(self.uffd.as_raw_fd(), PollFlags::POLLIN);

        // Loop, handling incoming events on the userfaultfd file descriptor.
        loop {
            // See what poll() tells us about the userfaultfd.
            let nready = poll(&mut [pollfd], -1).expect("Failed to poll");

            let revents = pollfd.revents().unwrap();
            println!(
                "poll() returns: nready = {}; POLLIN = {}; POLLERR = {}",
                nready,
                revents.contains(PollFlags::POLLIN),
                revents.contains(PollFlags::POLLERR),
            );

            // Read an event from the userfaultfd.
            let event = self
                .uffd
                .read_event()
                .expect("Failed to read uffd_msg")
                .expect("uffd_msg not ready");

            // We expect to receive either a Page Fault or Removed
            // event (if the balloon device is enabled).
            match event {
                userfaultfd::Event::Pagefault { addr, .. } => self.serve_pf(addr as *mut u8),
                userfaultfd::Event::Remove { start, end } => self.update_mem_state_mappings(
                    start as *mut u8 as u64,
                    end as *mut u8 as u64,
                    &MemPageState::Removed,
                ),
                _ => panic!("Unexpected event on userfaultfd"),
            }
        }
    }
}

fn get_peer_process_credentials(stream: UnixStream) -> libc::ucred {
    let mut creds: libc::ucred = libc::ucred {
        pid: 0,
        gid: 0,
        uid: 0,
    };
    let mut creds_size = mem::size_of::<libc::ucred>() as u32;

    let ret = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut creds as *mut _ as *mut _,
            &mut creds_size as *mut libc::socklen_t,
        )
    };
    if ret != 0 {
        panic!("Failed to get peer process credentials");
    }

    creds
}

fn create_mem_regions(mappings: &Vec<GuestRegionUffdMapping>) -> Vec<MemRegion> {
    let page_size = get_page_size().unwrap();
    let mut mem_regions: Vec<MemRegion> = Vec::with_capacity(mappings.len());

    for r in mappings.iter() {
        let mapping = r.clone();
        let mut addr = r.base_host_virt_addr;
        let end_addr = r.base_host_virt_addr + r.size as u64;
        let mut page_states = HashMap::new();

        while addr < end_addr {
            page_states.insert(addr, MemPageState::Uninitialized);
            addr += page_size as u64;
        }
        mem_regions.push(MemRegion {
            mapping,
            page_states,
        });
    }

    mem_regions
}

fn main() {
    let uffd_sock_path = std::env::args().nth(1).expect("No socket path given");
    let mem_file_path = std::env::args().nth(2).expect("No memory file given");

    let file = File::open(mem_file_path).expect("Cannot open memfile");
    let size = file.metadata().unwrap().len() as usize;

    // mmap a memory area used to bring in the faulting regions.
    let memfile_buffer = unsafe {
        mmap(
            ptr::null_mut(),
            size,
            ProtFlags::PROT_READ,
            MapFlags::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
        .expect("mmap failed")
    } as *const u8;

    // Get Uffd from UDS. We'll use the uffd to handle PFs for Firecracker.
    let listener = UnixListener::bind(&uffd_sock_path).expect("Cannot bind to socket path");

    let (stream, _) = listener.accept().expect("Cannot listen on UDS socket");
    let mut uffd_handler = UffdPfHandler::from_unix_stream(stream, memfile_buffer, size);

    uffd_handler.run_loop();
    println!("Uffd thread done!");
}
