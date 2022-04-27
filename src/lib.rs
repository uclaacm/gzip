//! An [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)-correct description
//! of the gzip file format in Rust.
//! 
//! To use the library, create a [Reader] or [Writer] with an underlying data structure
//! implementing [Read] or [Write], respectively. Libz stream initialization and management
//! is carried out automatically.

use std::{
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
#[derive(Debug, Clone, Copy)]
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

/// Lightweight reference to z_streamp for the given [Stream].
impl AsMut<z_streamp> for Stream {
    fn as_mut(&mut self) -> &mut z_streamp {
        // &mut addr_of_mut!(self.stream);
        todo!()
    }
}

impl AsRef<z_streamp> for Stream {
    fn as_ref(&self) -> &z_streamp {
        // self.into()
        todo!()
    }
}

impl Stream {
    fn default_stream() -> z_stream {
        unsafe {
            z_stream {
                zalloc: mem_alloc,
                zfree: mem_free,
                opaque: null_mut(),
                next_in: todo!(),
                avail_in: todo!(),
                total_in: todo!(),
                next_out: todo!(),
                avail_out: todo!(),
                total_out: todo!(),
                msg: todo!(),
                state: todo!(),
                data_type: todo!(),
                adler: todo!(),
                reserved: todo!(),
            }
        }
    }

    fn new(mode: Mode) -> Self {
        Self {
            mode,
            stream: Self::default_stream(),
        }
    }
}

pub struct Reader<R: Read> {
    /// Associated Zlib compression stream.
    stream: libz_sys::z_streamp,

    /// Underlying file, either read or write.
    file: R,
}

impl<R> Read for Reader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            inflate(self.stream, 1);
            todo!()
        }
    }
}

pub struct Writer<W: Write> {
    /// Associated Zlib compression stream.
    stream: Stream,

    /// Underlying file, either read or write.
    file: W,
}

impl<W> Write for Writer<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl<W> Writer<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            stream: Stream::new(Mode::DEFLATE),
            file: writer,
        }
    }
}
