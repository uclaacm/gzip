//! An [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)-correct description
//! of the gzip file format in Rust.
//!
//! To use the library, create a [Reader] or [Writer] with an underlying data structure
//! implementing [Read] or [Write], respectively. Libz stream initialization and management
//! is carried out automatically.

use std::{
    ffi::CString,
    io::{self, Read, Write},
    ptr::null_mut,
};

use libc::c_void;
use libz_sys::*;

/// Custom memory allocation handler for libz.
unsafe extern "C" fn mem_alloc(_opaque: *mut c_void, _val: u32, size: u32) -> *mut c_void {
    libc::malloc(size as usize)
}

/// Custom memory deallocation handler for libz.
unsafe extern "C" fn mem_free(_opaque: *mut c_void, ptr: *mut c_void) {
    libc::free(ptr)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    INFLATE,
    DEFLATE,
}

/// Safety-wrapped representation of a libz stream.
struct Stream {
    mode: Mode,
    stream: z_stream,
}

/// Opaque data passed between calls to malloc and free by the system zlib.
#[repr(C)]
#[derive(Debug, Clone)]
struct Opaque;

impl Opaque {
    /// Get the *mut c_void pointer for the given [Opaque].
    unsafe fn into_void(mut self) -> *mut c_void {
        &mut self as *mut _ as *mut c_void
    }
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
        z_stream {
            zalloc: mem_alloc,
            zfree: mem_free,
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

    fn new(mode: Mode) -> Self {
        Self {
            mode,
            stream: Self::default_stream(),
        }
    }

    fn init_inflate(&mut self, version: &[i8], stream_size: i32) -> io::Result<()> {
        assert_eq!(self.mode, Mode::INFLATE);
        unsafe {
            let res = inflateInit_(&mut self.stream as _, version.as_ptr(), stream_size);
            if res == Z_OK {
                Ok(())
            } else {
                Err(io::ErrorKind::Other.into())
            }
        }
    }

    fn init_deflate(&mut self, level: i32, version: &str, stream_size: i32) -> io::Result<()> {
        assert_eq!(self.mode, Mode::DEFLATE);
        unsafe {
            let version = CString::new(version).expect("string");
            let res = deflateInit_(&mut self.stream as _, level, version.as_ptr(), stream_size);
            if res == Z_OK {
                Ok(())
            } else {
                Err(io::ErrorKind::Other.into())
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
    fn new(file: R, buf_len: usize, version: &[i8], stream_size: i32) -> io::Result<Self> {
        let mut stream = Stream::new(Mode::INFLATE);
        stream.init_inflate(version, stream_size)?;

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
    pub fn new(
        writer: W,
        buf_len: usize,
        level: i32,
        version: &str,
        stream_size: i32,
    ) -> io::Result<Self> {
        let mut stream = Stream::new(Mode::DEFLATE);
        stream.init_deflate(level, version, stream_size)?;

        Ok(Self {
            stream,
            file: writer,
            buf: vec![0; buf_len],
        })
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, ffi::CStr, mem::size_of};

    use super::*;

    struct MockFile<W: Write>(RefCell<W>);

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
    fn write_smoke() {
        let output = RefCell::new(vec![]);
        // unsafe { println!("{:?}", CStr::from_ptr(zlibVersion()).to_str().unwrap()); }
        let stream_size = size_of::<Stream>() as i32;
        let mut gzip_writer =
            Writer::new(MockFile(output.clone()), 1024, 6, "1.2.11", stream_size).expect("writer");
        gzip_writer.write_all(b"test string").expect("write");
        assert_ne!(output.borrow().len(), 0);
    }
}
