// Copyright 2024 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::os::fd::AsRawFd;

use libc::{c_int, c_void, iovec, off_t, size_t};
use memfd;

use super::queue::FIRECRACKER_MAX_QUEUE_SIZE;
use crate::arch::PAGE_SIZE;

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum IovDequeError {
    /// Error with memfd: {0}
    Memfd(#[from] memfd::Error),
    /// Error while resizing memfd: {0}
    MemfdResize(std::io::Error),
    /// Error calling mmap: {0}
    Mmap(std::io::Error),
}

/// ['IovDeque'] is a ring buffer tailored for `struct iovec` objects.
///
/// From the point of view of API, [`IovDeque`] is a typical ring buffer that allows us to push
/// `struct iovec` objects at the end of the buffer and pop them from its beginning.
///
/// It is tailored to store `struct iovec` objects that described memory that was passed to us from
/// the guest via a VirtIO queue. This allows us to assume the maximum size of a ring buffer (the
/// negotiated size of the queue).
// An important feature of the data structure is that it can give us a slice of all `struct iovec`
// objects in the queue, so that we can use this `&mut [iovec]` to perform operations such as
// `readv`. A typical implementation of a ring buffer allows for entries to wrap around the end of
// the underlying buffer. For example, a ring buffer with a capacity of 10 elements which
// currently holds 4 elements can look like this:
//
//                      tail                        head
//                       |                           |
//                       v                           v
//                 +---+---+---+---+---+---+---+---+---+---+
// ring buffer:    | C | D |   |   |   |   |   |   | A | B |
//                 +---+---+---+---+---+---+---+---+---+---+
//
// When getting a slice for this data we should get something like that: &[A, B, C, D], which
// would require copies in order to make the elements continuous in memory.
//
// In order to avoid that and make the operation of getting a slice more efficient, we implement
// the optimization described in the "Optimization" section of the "Circular buffer" wikipedia
// entry: https://en.wikipedia.org/wiki/Circular_buffer. The optimization consists of allocating
// double the size of the virtual memory required for the buffer and map both parts on the same
// physical address. Looking at the same example as before, we should get, this picture:
//
//                                    head   |    tail
//                                     |     |     |
//                                     v     |     v
//   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//   | C | D |   |   |   |   |   |   | A | B | C | D |   |   |   |   |   |   | A | B |
//   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//            First virtual page             |       Second virtual page
//                                           |
//                                           |
//
//                                     Virtual memory
// ---------------------------------------------------------------------------------------
//                                    Physical memory
//
//                      +---+---+---+---+---+---+---+---+---+---+
//                      | C | D |   |   |   |   |   |   | A | B |
//                      +---+---+---+---+---+---+---+---+---+---+
//
// Like that, the elements stored in the buffer are always laid out in contiguous virtual memory,
// so making a slice out of them does not require any copies.
#[derive(Debug)]
pub struct IovDeque {
    pub iov: *mut libc::iovec,
    pub start: u16,
    pub len: u16,
}

// SAFETY: This is `Send`. We hold sole ownership of the underlying buffer.
unsafe impl Send for IovDeque {}

impl IovDeque {
    /// Create a [`memfd`] object that represents a single physical page
    fn create_memfd() -> Result<memfd::Memfd, IovDequeError> {
        // Create a sealable memfd.
        let opts = memfd::MemfdOptions::default().allow_sealing(true);
        let mfd = opts.create("sized-1K")?;

        // Resize to system page size.
        mfd.as_file()
            .set_len(PAGE_SIZE.try_into().unwrap())
            .map_err(IovDequeError::MemfdResize)?;

        // Add seals to prevent further resizing.
        mfd.add_seals(&[memfd::FileSeal::SealShrink, memfd::FileSeal::SealGrow])?;

        // Prevent further sealing changes.
        mfd.add_seal(memfd::FileSeal::SealSeal)?;

        Ok(mfd)
    }

    /// A safe wrapper on top of libc's `mmap` system call
    ///
    /// # Safety: Callers need to make sure that the arguments to `mmap` are valid
    unsafe fn mmap(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> Result<*mut c_void, IovDequeError> {
        let ptr = libc::mmap(addr, len, prot, flags, fd, offset);
        if ptr == libc::MAP_FAILED {
            return Err(IovDequeError::Mmap(std::io::Error::last_os_error()));
        }

        Ok(ptr)
    }

    /// Allocate memory for our ring buffer
    ///
    /// This will allocate exactly two pages of virtual memory. In order to implement the
    /// optimization that allows us to always have elements in contiguous memory we need
    /// allocations at the granularity of `PAGE_SIZE`. Now, our queues are at maximum 256
    /// descriptors long and `struct iovec` looks like this:
    ///
    /// ```Rust
    /// pub struct iovec {
    ///    pub iov_base: *mut ::c_void,
    ///    pub iov_len: ::size_t,
    /// }
    /// ```
    ///
    /// so, it's 16 bytes long. As a result, we need a single page for holding the actual data of
    /// our buffer.
    fn allocate_ring_buffer_memory() -> Result<*mut c_void, IovDequeError> {
        // The fact that we allocate two pages is due to the size of `struct iovec` times our queue
        // size equals the page size. Add here a debug assertion to reflect that and ensure that we
        // will adapt our logic if the assumption changes in the future.
        const {
            assert!(
                std::mem::size_of::<iovec>() * FIRECRACKER_MAX_QUEUE_SIZE as usize == PAGE_SIZE
            );
        }

        // SAFETY: We are calling the system call with valid arguments
        unsafe {
            Self::mmap(
                std::ptr::null_mut(),
                PAGE_SIZE * 2,
                libc::PROT_NONE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        }
    }

    /// Create a new [`IovDeque`] that can hold memory described by a single VirtIO queue.
    pub fn new() -> Result<Self, IovDequeError> {
        let memfd = Self::create_memfd()?;
        let raw_memfd = memfd.as_file().as_raw_fd();
        let buffer = Self::allocate_ring_buffer_memory()?;

        // Map the first page of virtual memory to the physical page described by the memfd object
        // SAFETY: We are calling the system call with valid arguments
        let _ = unsafe {
            Self::mmap(
                buffer,
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                raw_memfd,
                0,
            )
        }?;

        // Map the second page of virtual memory to the physical page described by the memfd object
        //
        // SAFETY: This is safe because:
        // * Both `buffer` and the result of `buffer.add(PAGE_SIZE)` are within bounds of the
        //   allocation we got from `Self::allocate_ring_buffer_memory`.
        // * The computed offset is `PAGE_SIZE * size_of::<c_void>() == PAGE_SIZE bytes` which fits
        //   in `isize`
        // * The resulting pointer is the beginning of the second page of our allocation, so it
        //   doesn't wrap around the address space.
        let next_page = unsafe { buffer.add(PAGE_SIZE) };

        // SAFETY: We are calling the system call with valid arguments
        let _ = unsafe {
            Self::mmap(
                next_page,
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                raw_memfd,
                0,
            )
        }?;

        Ok(Self {
            iov: buffer.cast(),
            start: 0,
            len: 0,
        })
    }

    /// Returns the number of `iovec` objects currently in the [`IovDeque`]
    #[inline(always)]
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if the [`IovDeque`] is full, `false` otherwise
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len() == FIRECRACKER_MAX_QUEUE_SIZE
    }

    /// Resets the queue, dropping all its elements.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.start = 0;
        self.len = 0;
    }

    /// Adds an `iovec` in the ring buffer.
    ///
    /// Returns an `IovDequeError::Full` error if the buffer is full.
    pub fn push_back(&mut self, iov: iovec) {
        // This should NEVER happen, since our ring buffer is as big as the maximum queue size.
        // We also check for the sanity of the VirtIO queues, in queue.rs, which means that if we
        // ever try to add something in a full ring buffer, there is an internal bug in the device
        // emulation logic. Panic here because the device is hopelessly broken.
        assert!(
            !self.is_full(),
            "The number of `iovec` objects is bigger than the available space"
        );

        // SAFETY: self.iov is a valid pointer and `self.start + self.len` is within range (we
        // asserted before that the buffer is not full).
        unsafe {
            self.iov
                .add((self.start + self.len) as usize)
                .write_volatile(iov)
        };
        self.len += 1;
    }

    /// Pops the first `nr_iovecs` iovecs from the buffer.
    ///
    /// Returns the total number of bytes of all the popped iovecs. This will panic if we are asked
    /// to pop more iovecs than what is currently available in the buffer.
    pub fn pop_front(&mut self, nr_iovecs: u16) {
        assert!(
            self.len() >= nr_iovecs,
            "Internal bug! Trying to drop more iovec objects than what is available"
        );

        self.start += nr_iovecs;
        self.len -= nr_iovecs;
        if self.start >= FIRECRACKER_MAX_QUEUE_SIZE {
            self.start -= FIRECRACKER_MAX_QUEUE_SIZE;
        }
    }

    /// Get a slice of the iovec objects currently in the buffer.
    pub fn as_slice(&self) -> &[iovec] {
        // SAFETY: Here we create a slice out of the existing elements in the buffer (not the whole
        // allocated memory). That means that we can:
        // * We can read `self.len * mem::size_of::<iovec>()` bytes out of the memory range we are
        //   returning.
        // * `self.iov.add(self.start.into())` is a non-null pointer and aligned.
        // * The underlying memory comes from a single allocation.
        // * The returning pointer points to `self.len` consecutive initialized `iovec` objects.
        // * We are only accessing the underlying memory through the returned slice. Since we are
        //   returning a slice of only the existing pushed elements the slice does not contain any
        //   aliasing references.
        // * The slice can be up to 1 page long which is smaller than `isize::MAX`.
        unsafe {
            let slice_start = self.iov.add(self.start.into());
            std::slice::from_raw_parts(slice_start, self.len.into())
        }
    }

    /// Get a mutable slice of the iovec objects currently in the buffer.
    pub fn as_mut_slice(&mut self) -> &mut [iovec] {
        // SAFETY: Here we create a slice out of the existing elements in the buffer (not the whole
        // allocated memory). That means that we can:
        // * We can read/write `self.len * mem::size_of::<iovec>()` bytes out of the memory range we
        //   are returning.
        // * The underlying memory comes from a single allocation.
        // * `self.iov.add(self.start.into())` is a non-null pointer and aligned
        // * The returning pointer points to `self.len` consecutive initialized `iovec` objects.
        // * We are only accessing the underlying memory through the returned slice. Since we are
        //   returning a slice of only the existing pushed elements the slice does not contain any
        //   aliasing references.
        // * The slice can be up to 1 page long which is smaller than `isize::MAX`.
        unsafe {
            let slice_start = self.iov.add(self.start.into());
            std::slice::from_raw_parts_mut(slice_start, self.len.into())
        }
    }
}

impl Drop for IovDeque {
    fn drop(&mut self) {
        // SAFETY: We are passing an address that we got from a previous allocation of `2 *
        // PAGE_SIZE` bytes by calling mmap
        let _ = unsafe { libc::munmap(self.iov.cast(), PAGE_SIZE * 2) };
    }
}

#[cfg(test)]
mod tests {
    use libc::iovec;

    use super::IovDeque;

    #[test]
    fn test_new() {
        let deque = IovDeque::new().unwrap();
        assert_eq!(deque.len(), 0);
    }

    fn make_iovec(id: u16, len: u16) -> iovec {
        iovec {
            iov_base: id as *mut libc::c_void,
            iov_len: len as usize,
        }
    }

    #[test]
    #[should_panic]
    fn test_push_back_too_many() {
        let mut deque = IovDeque::new().unwrap();
        assert_eq!(deque.len(), 0);

        for i in 0u16..256 {
            deque.push_back(make_iovec(i, i));
            assert_eq!(deque.len(), i + 1);
        }

        deque.push_back(make_iovec(0, 0));
    }

    #[test]
    #[should_panic]
    fn test_pop_front_from_empty() {
        let mut deque = IovDeque::new().unwrap();
        deque.pop_front(1);
    }

    #[test]
    #[should_panic]
    fn test_pop_front_too_many() {
        let mut deque = IovDeque::new().unwrap();
        deque.push_back(make_iovec(42, 42));
        deque.pop_front(2);
    }

    #[test]
    fn test_pop() {
        let mut deque = IovDeque::new().unwrap();
        assert_eq!(deque.len(), 0);
        assert!(!deque.is_full());
        deque.pop_front(0);

        for i in 0u16..256 {
            deque.push_back(make_iovec(i, i));
            assert_eq!(deque.len(), i + 1);
        }

        assert!(deque.is_full());
        assert!(deque.len() != 0);

        for i in 0u16..256 {
            deque.pop_front(1);
            assert_eq!(deque.len(), 256 - i - 1);
        }
    }

    #[test]
    fn test_pop_many() {
        let mut deque = IovDeque::new().unwrap();

        for i in 0u16..256 {
            deque.push_back(make_iovec(i, i));
        }

        deque.pop_front(1);
        assert_eq!(deque.len(), 255);
        deque.pop_front(2);
        assert_eq!(deque.len(), 253);
        deque.pop_front(4);
        assert_eq!(deque.len(), 249);
        deque.pop_front(8);
        assert_eq!(deque.len(), 241);
        deque.pop_front(16);
        assert_eq!(deque.len(), 225);
        deque.pop_front(32);
        assert_eq!(deque.len(), 193);
        deque.pop_front(64);
        assert_eq!(deque.len(), 129);
        deque.pop_front(128);
        assert_eq!(deque.len(), 1);
    }

    #[test]
    fn test_as_slice() {
        let mut deque = IovDeque::new().unwrap();
        assert!(deque.as_slice().is_empty());

        for i in 0..256 {
            deque.push_back(make_iovec(i, 100));
            assert_eq!(deque.as_slice().len(), (i + 1) as usize);
        }
        let copy: Vec<iovec> = deque.as_slice().to_vec();

        assert_eq!(copy.len(), deque.len() as usize);
        for (i, iov) in deque.as_slice().iter().enumerate() {
            assert_eq!(iov.iov_len, copy[i].iov_len);
        }
    }

    #[test]
    fn test_as_mut_slice() {
        let mut deque = IovDeque::new().unwrap();
        assert!(deque.as_mut_slice().is_empty());

        for i in 0..256 {
            deque.push_back(make_iovec(i, 100));
            assert_eq!(deque.as_mut_slice().len(), (i + 1) as usize);
        }

        let copy: Vec<iovec> = deque.as_mut_slice().to_vec();
        deque
            .as_mut_slice()
            .iter_mut()
            .for_each(|iov| iov.iov_len *= 2);

        assert_eq!(copy.len(), deque.len() as usize);
        for (i, iov) in deque.as_slice().iter().enumerate() {
            assert_eq!(iov.iov_len, 2 * copy[i].iov_len);
        }
    }
}
