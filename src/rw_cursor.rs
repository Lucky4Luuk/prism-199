use std::io::{self, ErrorKind, IoSliceMut, ReadBuf, prelude::*};

pub struct RwCursor<T> {
    inner: T,
    pos_w: u64,
    pos_r: u64,
}

impl<T> RwCursor<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: inner,
            pos_w: 0,
            pos_r: 0,
        }
    }
}

impl<T> RwCursor<T>
where
    T: AsRef<[u8]>,
{
    pub fn remaining_slice_read(&self) -> &[u8] {
        let len = self.pos_r.min(self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(len as usize)..]
    }

    pub fn is_empty_read(&self) -> bool {
        self.pos_r >= self.inner.as_ref().len() as u64
    }
}

impl<T> Clone for RwCursor<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        RwCursor {
            inner: self.inner.clone(),
            pos_w: self.pos_w,
            pos_r: self.pos_r,
        }
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
        self.pos_w = other.pos_w;
        self.pos_r = other.pos_r;
    }
}

impl<T> BufRead for RwCursor<T>
where
    T: AsRef<[u8]>,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.remaining_slice_read())
    }
    fn consume(&mut self, amt: usize) {
        self.pos_r += amt as u64;
    }
}

impl<T> Read for RwCursor<T>
where
    T: AsRef<[u8]>
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut self.remaining_slice_read(), buf)?;
        self.pos_r += n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, buf: &mut ReadBuf<'_>) -> io::Result<()> {
        let prev_filled = buf.filled_len();

        Read::read_buf(&mut self.fill_buf()?, buf)?;

        self.pos_r += (buf.filled_len() - prev_filled) as u64;

        Ok(())
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            nread += n;
            if n < buf.len() {
                break;
            }
        }
        Ok(nread)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let n = buf.len();
        Read::read_exact(&mut self.remaining_slice_read(), buf)?;
        self.pos_r += n as u64;
        Ok(())
    }
}

#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<usize> {
    let pos = std::cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

fn vec_write<A>(pos_mut: &mut u64, vec: &mut Vec<u8, A>, buf: &[u8]) -> io::Result<usize>
where
    A: std::alloc::Allocator,
{
    let pos: usize = (*pos_mut).try_into().map_err(|_| {ErrorKind::InvalidInput})?;
    // Make sure the internal buffer is as least as big as where we
    // currently are
    let len = vec.len();
    if len < pos {
        // use `resize` so that the zero filling is as efficient as possible
        vec.resize(pos, 0);
    }
    // Figure out what bytes will be used to overwrite what's currently
    // there (left), and what will be appended on the end (right)
    {
        let space = vec.len() - pos;
        let (left, right) = buf.split_at(std::cmp::min(space, buf.len()));
        vec[pos..pos + left.len()].copy_from_slice(left);
        vec.extend_from_slice(right);
    }

    // Bump us forward
    *pos_mut = (pos + buf.len()) as u64;
    Ok(buf.len())
}

impl Write for RwCursor<&mut [u8]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos_w, self.inner, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<A> Write for RwCursor<&mut Vec<u8, A>>
where
    A: std::alloc::Allocator,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        vec_write(&mut self.pos_w, self.inner, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<A> Write for RwCursor<Vec<u8, A>>
where
    A: std::alloc::Allocator,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        vec_write(&mut self.pos_w, &mut self.inner, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
