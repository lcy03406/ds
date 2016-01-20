use std::io::Write;

pub trait BufWrite : Write {
    fn reserve_buf(&mut self, min_size : usize) -> &mut [u8];
    fn buf_filled(&mut self, amt: usize);
}
