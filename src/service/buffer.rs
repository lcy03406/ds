use std::ptr;
use std::cmp;
use std::io::{Result, Read, Write, BufRead};
use super::bufwrite::BufWrite;
// A dumb memcpy buffer.
pub struct Buffer {
    read : usize,
    write : usize,
    buf : Vec<u8>,
}

impl Buffer {
    pub fn with_capacity(cap : usize) -> Buffer {
        let mut v = Vec::with_capacity(cap);
        unsafe { v.set_len(cap); }
        Buffer {
            read : 0,
            write : 0,
            buf : v
        }
    }

    fn data_slice(&self) -> &[u8] {
        &self.buf[self.read..self.write]
    }

    fn space_slice(&mut self) -> &mut [u8] {
        &mut self.buf[self.write..]
    }

    fn cap(&self) -> usize {
        self.buf.len()
    }

    pub fn data_len(&self) -> usize {
        self.write - self.read
    }

    fn space_len(&self) -> usize {
        self.cap() - self.write
    }

    pub fn is_empty(&self) -> bool {
        self.data_len() == 0
    }

    fn skip_read(&mut self, len : usize) {
        assert!(len <= self.data_len());
        self.read += len;
        if self.is_empty() {
            self.write = 0;
            self.read = 0;
        } else if self.read + self.read >= self.cap() {
            self.move_to_begin();
        }
    }

    fn done_write(&mut self, len : usize) {
        assert!(len <= self.space_len());
        self.write += len;
    }

    fn reserve(&mut self, cap : usize) {
        if self.buf.len() < cap {
            self.buf.reserve(cap);
            unsafe { self.buf.set_len(cap); }
        }
    }
    fn reserve_more(&mut self, more : usize) {
        let cap = self.write + more;
        self.reserve(cap);
    }
    unsafe fn ptr_read(&self) -> *const u8 {
        self.buf[..].as_ptr().offset(self.read as isize)
    }
    unsafe fn mut_ptr_buf(&mut self) -> *mut u8 {
        self.buf[..].as_mut_ptr()
    }
    unsafe fn mut_ptr_write(&mut self) -> *mut u8 {
        self.buf[..].as_mut_ptr().offset(self.write as isize)
    }
    fn move_to_begin(&mut self) {
        unsafe {
            ptr::copy(self.ptr_read(), self.mut_ptr_buf(), self.write - self.read);
            self.write -= self.read;
            self.read = 0;
        }
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        debug_assert!(self.write >= self.read);
        debug_assert!(self.cap() >= self.write);
        if self.write == self.read {
            Ok(0)
        } else  {
            let len = cmp::min(self.write - self.read, buf.len());
            unsafe {
                ptr::copy_nonoverlapping(self.ptr_read(), buf.as_mut_ptr(), len);
            }
            self.skip_read(len);
            Ok(len)
        }
    }
}

impl BufRead for Buffer {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        Ok(self.data_slice())
    }
    fn consume(&mut self, amt: usize) {
        self.skip_read(amt);
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let space = self.space_len();
        let len = buf.len();
        if space < len {
            self.move_to_begin();
            self.reserve_more(len);
        }
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), self.mut_ptr_write(), len);
        }
        self.done_write(len);
        Ok(len)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl BufWrite for Buffer {
    fn reserve_buf(&mut self, min_size : usize) -> &mut [u8] {
        let space = self.space_len();
        if space < min_size {
            self.move_to_begin();
            self.reserve_more(min_size);
        }
        self.space_slice()
    }
    fn buf_filled(&mut self, amt: usize) {
        self.done_write(amt);
    }
}
