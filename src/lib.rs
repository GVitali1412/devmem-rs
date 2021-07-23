// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::ptr;

/// Mapping between the virtual and physical address space
pub struct Mapping {
    map_base: *mut libc::c_void,
    len: libc::size_t,
    slice_base: *mut u8,
    slice_max_len: libc::size_t,
}

impl Mapping {
    /// Create a new mapping of `len` bytes, starting at `physical_addr`
    pub unsafe fn new(physical_addr: usize, len: usize) -> std::io::Result<Mapping> {
        assert!(len > 0, "The mapping length must be greater than 0");

        // mmap() can map a file only at an offset that is a multiple of the
        // page size
        // Compute the frame offset of the physical address and the starting
        // address of the corresponding page frame
        let page_size = libc::sysconf(libc::_SC_PAGESIZE) as usize;
        let frame_offset = physical_addr % page_size;
        let frame_addr = physical_addr - frame_offset;

        // Open /dev/mem with O_RDWR and O_SYNC flags
        let devmem_file = OpenOptions::new()
            .write(true)
            .read(true)
            .custom_flags(libc::O_RDWR | libc::O_SYNC)
            .open("/dev/mem")?;

        let devmem_fd = devmem_file.as_raw_fd();

        // Mmap /dev/mem in the virtual address space, starting at offset in
        // the file equal to the frame address
        // map_base points to the virtual address mapped to frame_addr
        let map_base = libc::mmap(
            ptr::null_mut(),
            len + frame_offset,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            devmem_fd,
            frame_addr as libc::off_t,
        );

        // slice_base points to the virtual address mapped to physical_addr
        let slice_base = (map_base as *mut u8).add(frame_offset);

        Ok(Mapping {
            map_base,
            len: len + frame_offset,
            slice_base,
            slice_max_len: len,
        })
    }

    /// Copy a slice of bytes from the physical address space into `dst`
    pub fn copy_into_slice(&self, dst: &mut [u8]) {
        assert!(self.slice_max_len >= dst.len());
        let mapped_slice = self.as_slice(dst.len());
        dst.copy_from_slice(mapped_slice);
    }

    /// Copy a slice of bytes from `src` to the physical address space
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert!(self.slice_max_len >= src.len());
        let mapped_slice = self.as_mut_slice(src.len());
        mapped_slice.copy_from_slice(src);
    }

    fn as_slice(&self, slice_len: usize) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.slice_base, slice_len) }
    }

    fn as_mut_slice(&mut self, slice_len: usize) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.slice_base, slice_len) }
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        let _ = unsafe { libc::munmap(self.map_base, self.len) };
    }
}

/// Copy a slice of bytes from the physical address space, starting at `physical_addr`, into `dst`
pub unsafe fn read_into_slice(physical_addr: usize, dst: &mut [u8]) -> std::io::Result<()> {
    let map = Mapping::new(physical_addr, dst.len())?;
    map.copy_into_slice(dst);
    Ok(())
}

/// Copy a slice of bytes from `src` into the physical address space, starting at `physical_addr`
pub unsafe fn write_from_slice(physical_addr: usize, src: &[u8]) -> std::io::Result<()> {
    let mut map = Mapping::new(physical_addr, src.len())?;
    map.copy_from_slice(src);
    Ok(())
}
