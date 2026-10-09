// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Buffers the SDK reads frames' bitstreams into.

use std::alloc::{ alloc_zeroed, dealloc, handle_alloc_error, Layout };
use std::ptr::NonNull;

/// The alignment of a buffer the SDK reads a frame's bitstream into, and the
/// granularity of its length: the SDK reads straight from the file into it, as
/// unbuffered I/O does, and fails with [`BrawError::Fail`](crate::BrawError::Fail)
/// otherwise.
pub const BIT_STREAM_ALIGNMENT: usize = 4096;

/// A zeroed buffer to read a frame's bitstream into, laid out as the SDK requires:
/// [`BIT_STREAM_ALIGNMENT`]-aligned, its length a multiple of it.
///
/// See [`BlackmagicRawClipEx::read_frame`](crate::BlackmagicRawClipEx::read_frame).
pub struct BitStreamBuffer {
    ptr: NonNull<u8>,
    len: usize,
}

// SAFETY: the buffer owns its bytes, which are plain data.
unsafe impl Send for BitStreamBuffer {}
// SAFETY: as above; `&BitStreamBuffer` only reads them.
unsafe impl Sync for BitStreamBuffer {}

impl BitStreamBuffer {
    /// A buffer with room for `len` bytes, its length rounded up to a multiple of
    /// [`BIT_STREAM_ALIGNMENT`] — for a frame, its `bit_stream_size_bytes`; for any
    /// frame of a clip, `max_bit_stream_size_bytes`.
    ///
    /// # Panics
    /// If the rounded length overflows `isize`.
    pub fn new(len: usize) -> Self {
        let len = len.max(1).checked_next_multiple_of(BIT_STREAM_ALIGNMENT).expect("bit stream buffer length overflows");
        let layout = Layout::from_size_align(len, BIT_STREAM_ALIGNMENT).expect("bit stream buffer length overflows");
        // SAFETY: the layout has a non-zero size.
        let ptr = NonNull::new(unsafe { alloc_zeroed(layout) }).unwrap_or_else(|| handle_alloc_error(layout));
        Self { ptr, len }
    }

    fn layout(&self) -> Layout {
        // SAFETY: `new` checked this layout.
        unsafe { Layout::from_size_align_unchecked(self.len, BIT_STREAM_ALIGNMENT) }
    }
}

impl Drop for BitStreamBuffer {
    fn drop(&mut self) {
        // SAFETY: allocated by `new` with this layout.
        unsafe { dealloc(self.ptr.as_ptr(), self.layout()) }
    }
}

impl std::ops::Deref for BitStreamBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        // SAFETY: `len` initialised bytes, owned by `self`.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl std::ops::DerefMut for BitStreamBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        // SAFETY: as in `deref`, borrowed uniquely.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl AsRef<[u8]> for BitStreamBuffer {
    fn as_ref(&self) -> &[u8] { self }
}

impl AsMut<[u8]> for BitStreamBuffer {
    fn as_mut(&mut self) -> &mut [u8] { self }
}

impl std::fmt::Debug for BitStreamBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitStreamBuffer").field("len", &self.len).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buffer_is_aligned_zeroed_and_rounded_up_to_whole_pages() {
        for (asked, len) in [(0, 4096), (1, 4096), (4096, 4096), (4097, 8192), (12_288, 12_288)] {
            let mut buffer = BitStreamBuffer::new(asked);
            assert_eq!(buffer.len(), len);
            assert_eq!(buffer.as_ptr() as usize % BIT_STREAM_ALIGNMENT, 0);
            assert!(buffer.iter().all(|&b| b == 0));
            buffer[len - 1] = 1;
        }
    }
}
