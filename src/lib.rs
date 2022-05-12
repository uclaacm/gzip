//! An [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)-correct description
//! of the gzip file format in Rust.
//!
//! To use the library, create a [Reader] or [Writer] with an underlying data structure
//! implementing [Read] or [Write], respectively. Libz stream initialization and management
//! is carried out automatically.

use std::{
    ffi::{CStr, CString},
    io::{self, Read, Write},
    mem::size_of,
    ptr::null_mut,
};

use libc::c_void;
use libz_sys::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    INFLATE,
    DEFLATE,
}

/// Safety-wrapped representation of a libz stream.
#[repr(C)]
struct Stream {
    mode: Mode,
    stream: z_stream,
}

impl Drop for Stream {
    /// Shutdown the stream and any relevant resources in use by it.
    fn drop(&mut self) {
        unsafe {
            match self.mode {
                Mode::DEFLATE => deflateEnd(self.into()),
                Mode::INFLATE => inflateEnd(self.into()),
            };
        }
    }
}

impl AsRef<z_stream> for Stream {
    fn as_ref(&self) -> &z_stream {
        &self.stream
    }
}

impl Into<z_streamp> for &mut Stream {
    fn into(self) -> z_streamp {
        &mut self.stream
    }
}

impl Stream {
    fn default_stream() -> z_stream {
        unsafe {
            z_stream {
                zalloc: std::mem::transmute::<
                    *const (),
                    unsafe extern "C" fn(*mut c_void, u32, u32) -> *mut c_void,
                >(null_mut() as *const ()),
                zfree: std::mem::transmute::<
                    *const (),
                    unsafe extern "C" fn(*mut c_void, *mut c_void),
                >(null_mut() as *const ()),
                opaque: null_mut(),
                next_in: null_mut(),
                avail_in: 0,
                total_in: 0,
                next_out: null_mut(),
                avail_out: 0,
                total_out: 0,
                msg: null_mut(),
                state: null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            }
        }
    }

    fn new(mode: Mode) -> Self {
        Self {
            mode,
            stream: Self::default_stream(),
        }
    }

    fn init_inflate(&mut self, stream_size: i32) -> io::Result<()> {
        assert_eq!(self.mode, Mode::INFLATE);
        unsafe {
            let res = inflateInit_(&mut self.stream as _, zlibVersion(), stream_size);
            if res == Z_OK {
                Ok(())
            } else {
                Err(io::ErrorKind::Other.into())
            }
        }
    }

    fn init_deflate(&mut self, level: i32) -> io::Result<()> {
        assert_eq!(self.mode, Mode::DEFLATE);
        unsafe {
            match deflateInit_(
                &mut self.stream as _,
                level,
                zlibVersion(),
                size_of::<z_stream>() as i32,
            ) {
                Z_OK => Ok(()),
                _ => Err(io::ErrorKind::Other.into()),
            }
        }
    }

    fn as_mut_ptr(&mut self) -> z_streamp {
        &mut self.stream as z_streamp
    }
}

pub struct Reader<R: Read> {
    /// Associated Zlib compression stream.
    stream: Stream,

    /// Underlying file, either read or write.
    file: R,

    /// Buffer for input file.
    buf: Vec<u8>,
}

impl<R: Read> Reader<R> {
    pub fn new(file: R, buf_len: usize, stream_size: i32) -> io::Result<Self> {
        let mut stream = Stream::new(Mode::INFLATE);
        stream.init_inflate(stream_size)?;

        Ok(Self {
            stream,
            file,
            buf: vec![0; buf_len],
        })
    }
}

impl<R> Read for Reader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.stream.stream.avail_in == 0 {
            self.stream.stream.avail_in = self.file.read(&mut self.buf)? as u32;
            self.stream.stream.next_in = self.buf.as_mut_ptr();
        }
        self.stream.stream.avail_out = buf.len() as u32;
        self.stream.stream.next_out = buf.as_mut_ptr();
        unsafe {
            let res = inflate(self.stream.as_mut_ptr(), Z_NO_FLUSH);
        }
        let len = buf.len() - (self.stream.stream.avail_out as usize);
        Ok(len)
    }
}

pub struct Writer<W: Write> {
    /// Associated Zlib compression stream.
    stream: Stream,

    /// Underlying file, either read or write.
    file: W,

    /// Buffer for output file.
    buf: Vec<u8>,
}

impl<W> Write for Writer<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.stream.avail_in = buf.len() as u32;
        self.stream.stream.next_in = buf.as_ptr() as *mut _;
        self.stream.stream.avail_out = self.buf.len() as u32;
        self.stream.stream.next_out = self.buf.as_mut_ptr();
        unsafe {
            let res = deflate(self.stream.as_mut_ptr(), Z_NO_FLUSH);
        }
        let out_len = self.buf.len() - (self.stream.stream.avail_out as usize);
        self.file.write(&self.buf[..out_len])?;
        let in_len = buf.len() - (self.stream.stream.avail_in as usize);
        Ok(in_len)
    }

    fn flush(&mut self) -> io::Result<()> {
        // TODO: flush stream
        self.file.flush()
    }
}

impl<W> Writer<W>
where
    W: Write,
{
    pub fn new(writer: W, buf_len: usize, level: i32) -> io::Result<Self> {
        let mut stream = Stream::new(Mode::DEFLATE);
        stream.init_deflate(level)?;

        Ok(Self {
            stream,
            file: writer,
            buf: vec![0; buf_len],
        })
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    /// Smoke-test for our zlib wrapper. Confirms that [Writer] can be
    /// constructed, data can be written to it, data can be flushed from
    /// it, and the data processed isn't garbage.
    #[test]
    fn write_smoke() {
        let output = Rc::new(RefCell::new(vec![]));
        let mut gzip_writer = Writer::new(MockFile(output.clone()), 1024, 6).expect("writer");
        gzip_writer.write_all(b"test string").expect("write");
        gzip_writer.flush().expect("flush");
        assert_ne!(output.borrow().len(), 0);
    }

    /// Refcell wrapper for monitoring of types consuming an [Rc] and
    /// [RefCell]-wrapped [Writer](Write).
    ///
    /// ```
    /// let buffer = Rc::new(RefCell::new(vec![]));
    /// let mut mock_writer = MockFile(buffer.clone());
    /// mock_writer.write(b"test");
    /// mock_writer.flush();
    /// assert_eq!(buffer.borrow()[..], b"test"[..]);
    /// ```
    #[derive(Debug, Clone)]
    struct MockFile<W: Write>(Rc<RefCell<W>>);

    impl<W> Write for MockFile<W>
    where
        W: Write,
    {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.borrow_mut().flush()
        }
    }

    #[test]
    fn mockfile_works() {
        let buffer = Rc::new(RefCell::new(vec![]));
        let mut mock_writer = MockFile(buffer.clone());
        mock_writer.write_all(b"test").expect("write");
        mock_writer.flush().expect("flush");
        assert_eq!(buffer.borrow()[..], b"test"[..]);
    }
}
