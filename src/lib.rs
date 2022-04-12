//! An [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)-correct description
//! of the gzip file format in Rust.

use std::{
    convert,
    io::{self, Read, Write},
    result,
};

use libz_sys::*;

/// Magic number identifying a gzip archive.
const GZIP_MAGIC: [u8; 2] = [0o037, 0o213];
/// Magic number for older gzip archives.
const OLD_GZIP_MAGIC: [u8; 2] = [0o37, 0o236];
const LZH_MAGIC: [u8; 2] = [0o037, 0o236];
const PKZIP_MAGIC: [u8; 4] = [0o120, 0o113, 0o003, 0o004];

/// OS codes used by the gzip format.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OsCode {
    FAT = 0,
    AMIGA = 1,
    VMS = 2,
    UNIX = 3,
    VMCMS = 4,
    ATARI = 5,
    HPFS = 6,
    MACINTOSH = 7,
    ZSYSTEM = 8,
    CPM = 9,
    TOPS = 10,

    /// Windows' preferred file system. The successor to FAT.
    NTFS = 11,

    QDOS = 12,
    ACORN = 13,

    /// Provided for historical reasons.
    UNKNOWN = 255,
}

/// A member in a gzip file.
///
/// A gzip file consists of a series of "members" (compressed data sets). The
/// format of each member is specified in the following data structure.
/// The members simply appear one after another in the file,
/// with no additional information before, between, or after them.
///
/// See: [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)
#[derive(Debug, Clone)]
pub struct Member {
    /// Method used in compressing this member.
    method: Method,

    /// One-hot byte enabling extra fields in a gzip member.
    flags: u8,

    /// Most recent modification time of the compressed file.
    mtime: u32,

    /// Provided if [Flag::ExtraField] is set.
    extra_field: Option<Subfield>,

    /// Zero-terminated string containing the original file name, if [Flag::Name] is set.
    name: Option<String>,

    /// Zero-terminated file comment, if [Flag::Comment] is set.
    comment: Option<String>,

    /// CRC-16 of the gzip header, provided if [Flag::HeaderCrc] is set.
    crc_16: Option<u16>,

    /// Data associated with this member.
    data: Vec<u8>,

    /// CRC-32 of the original, uncompressed file.
    crc_32: u32,

    /// Size of the original, uncompressed file mod 2^32.
    size: u32,
}

pub struct Archive<RW> {
    /// Whether the stream has been initialized.
    init: bool,

    /// Associated Zlib compression stream.
    stream: libz_sys::z_streamp,

    /// Underlying file, either read or write.
    file: RW,
}

impl Default for Member {
    fn default() -> Self {
        Self {
            method: Method::Deflate,
            flags: 0,
            mtime: 0,
            extra_field: None,
            name: None,
            comment: None,
            crc_16: None,
            data: vec![],
            crc_32: 0,
            size: 0,
        }
    }
}

/// If opened on a writer, then write headers and compress data.
impl<RW> Write for Archive<RW>
where
    RW: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

/// If opened on a reader, then read decompressed data.
impl<RW> Read for Archive<RW>
where
    RW: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

/// Unifying trait for DEFLATE stream reading and writing.
trait Deflatable<RW> {
    fn initialize_stream(&mut self) -> io::Result<usize>;
}

/// Deflate implementation.
impl<U> Archive<U>
where
    U: Write,
{
    /// Initialize the underlying libz stream for this [Archive].
    fn initialize_stream(&mut self) -> io::Result<usize> {
        unsafe {
            match deflateInit_(self.stream, 0, libz_sys::zlibVersion(), 4096) {
                Z_MEM_ERROR => Err(io::ErrorKind::OutOfMemory.into()),
                Z_STREAM_ERROR => Err(io::ErrorKind::Other.into()),
                Z_VERSION_ERROR => Err(io::ErrorKind::Other.into()),
                _ => Ok(0),
            }
        }
    }
}

impl<T, U> Deflatable<T> for Archive<U>
where
    T: Read,
{
    /// Initialize the underlying libz stream for this [Archive].
    fn initialize_stream(&mut self) -> io::Result<usize> {
        unsafe {
            match inflateInit_(self.stream, zlibVersion(), 4096) {
                Z_MEM_ERROR => Err(io::ErrorKind::OutOfMemory.into()),
                Z_STREAM_ERROR => Err(io::ErrorKind::Other.into()),
                Z_VERSION_ERROR => Err(io::ErrorKind::Other.into()),
                _ => Ok(0),
            }
        }
    }
}

/// Optional associated data to a [Member].
#[derive(Debug, Clone)]
pub struct Subfield {
    pub id: SubfieldId,
    len: u32,
    data: Vec<u8>,
}

/// Available subfield IDs.
#[derive(Debug, Clone, Copy)]
pub enum SubfieldId {
    Apollo,
}

impl Into<[u8; 2]> for SubfieldId {
    fn into(self) -> [u8; 2] {
        match self {
            Self::Apollo => [0x41, 0x70],
        }
    }
}

/// Compression methods
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Method {
    Store = 0,
    Compress = 1,
    Pack = 2,
    Lzh = 3,
    Deflate = 8,
    MaxMethods = 9,
}

impl Into<u8> for Method {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Gzip flag bytes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Flag {
    /// File probably ASCII text. (FTEXT)
    Ascii = 1,

    /// CRC16 for the gzip header. (FHCRC)
    HeaderCrc = 1 << 1,

    /// Extra field present. (FEXTRA)
    ExtraField = 1 << 2,

    /// Original file name present. (FNAME)
    OriginalName = 1 << 3,

    /// A zero-terminated file comment is present.
    Comment = 1 << 4,

    /// The highest three bits of the flag field are zero.
    Reserved = 0,
}

impl Into<u8> for Flag {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Custom(&'static str),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}
