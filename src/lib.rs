//! An [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952) correct implementation
//! of the gzip file format in Rust.

use std::{
    io::{self, Read, Write},
    result, time,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

/// Magic number for a gzip archive.
pub const GZIP_MAGIC: [u8; 2] = [0o037, 0o213];

/// Previously-used magic number for a gzip archive.
const OLD_GZIP_MAGIC: [u8; 2] = [0o37, 0o236];
const LZH_MAGIC: [u8; 2] = [0o037, 0o236];
const PKZIP_MAGIC: [u8; 4] = [0o120, 0o113, 0o003, 0o004];

/// OS codes used by the gzip format.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OsCode {
    /// The FAT file system.
    FAT = 0,

    AMIGA = 1,
    VMS = 2,

    /// EXT-like file systems.
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

/// Optional associated data to a [Member].
#[derive(Debug, Clone)]
pub struct Subfield {
    pub id: SubfieldId,

    /// Length of the subfield.
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

impl From<u8> for Method {
    fn from(i: u8) -> Self {
        match i {
            0 => Method::Store,
            1 => Method::Compress,
            2 => Method::Pack,
            3 => Method::Lzh,
            8 => Method::Deflate,
            _ => Method::MaxMethods,
        }
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

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

/// A member in a gzip file.
///
/// A gzip file consists of a series of "members" (compressed data sets). The format of each member is specified in the following
/// data structure.  The members simply appear one after another in the file,
/// with no additional information before, between, or after them.
///
/// See: [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)
#[derive(Debug)]
pub struct Member {
    /// Method used in compressing this member.
    method: Method,

    /// One-hot bit vector enabling extra fields in a gzip member.
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

impl<'a, V> From<V> for Member
where
    V: IntoIterator<Item = &'a u8>,
{
    fn from(v: V) -> Self {
        let mut m = Self::default();
        let mut it = v.into_iter().copied();
        m.method = Method::from(it.next().unwrap());
        m.flags = it.next().unwrap();
        m.mtime = u32::from_be_bytes([
            it.next().unwrap(),
            it.next().unwrap(),
            it.next().unwrap(),
            it.next().unwrap(),
        ]);

        if m.flags & Flag::ExtraField as u8 != 0 {
            m.extra_field = todo!();
        }

        if m.flags & Flag::OriginalName as u8 != 0 {
            m.name = todo!();
        }
        if m.flags & Flag::Comment as u8 != 0 {
            m.comment = todo!();
        }
        if m.flags & Flag::HeaderCrc as u8 != 0 {
            m.crc_16 = todo!();
            // TODO: confirm equality of header CRC
        }

        m.data = it.collect();
        let read = m.data.len();
        m.size = u32::from_be_bytes([
            m.data[read - 4],
            m.data[read - 3],
            m.data[read - 2],
            m.data[read - 1],
        ]);
        m.crc_32 = u32::from_be_bytes([
            m.data[read - 8],
            m.data[read - 7],
            m.data[read - 6],
            m.data[read - 5],
        ]);
        m.data.truncate(read - 8);
        m
    }
}

/// Format of a gzip archive.
pub type Archive = Vec<Member>;

/// Options to instantiate a gzip [Member].
#[derive(Debug)]
pub struct Options {
    /// Compression level to be used.
    level: Compression,

    /// Use most recent modification time of the compressed file
    /// (otherwise set to 0).
    mtime: Option<time::SystemTime>,

    /// Include an extra field.
    extra_field: Option<Subfield>,

    /// Include the original file name as a zero-terminated string.
    name: Option<String>,

    /// Include a zero-terminated file comment.
    comment: Option<String>,

    /// Take the CRC-16 of the gzip header when compressing.
    crc_16: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            level: Compression::fast(),
            mtime: None,
            extra_field: None,
            name: None,
            comment: None,
            crc_16: false,
        }
    }
}

impl Options {
    /// Set the level of compression to be used.
    pub fn level<'a>(&'a mut self, level: u32) -> &'a Self {
        self.level = Compression::new(level);
        self
    }

    /// Set the name of the original file.
    pub fn name<'a, S: ToString>(&'a mut self, name: &S) -> &'a Self {
        self.name = Some(name.to_string());
        self
    }

    /// Include an optional comment.
    pub fn comment<'a, S: ToString>(&'a mut self, comment: &S) -> &'a Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// Write a CRC-16 checksum of the archive header.
    pub fn crc_16<'a, S: AsRef<str>>(&'a mut self) -> &'a Self {
        self.crc_16 = true;
        self
    }

    /// Enable and set the extra field for this writer.
    pub fn extra<'a>(&'a mut self, subfield: &Subfield) -> &'a Self {
        self.extra_field = Some(subfield.clone());
        self
    }

    /// Build the [GzWriter] for this operation.
    pub fn to_writer<W: Write>(&mut self, writer: &mut W) -> GzWriter<W> {
        todo!()
    }
}

pub struct GzWriter<W: Write> {
    /// Whether or not the header has been written.
    header_written: bool,

    /// Number of bytes written. Used as the original file size.
    bytes_written: usize,

    /// Options supplied to the writer.
    options: Options,

    /// Output [writer](Write).
    output: GzEncoder<W>,
}

impl<W: Write> Write for GzWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.header_written {
            // Write header for the gzip archive.
            todo!()
        }
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl<W: Write> GzWriter<W> {
    fn new(options: Options) -> Self {
        todo!()
    }

    /// Terminate the current [GzWriter] file, writing the original file size and a CRC-32.
    pub fn finish(&mut self) -> io::Result<usize> {
        Ok(self.output.write(&self.bytes_written.to_le_bytes())? + self.output.write(b"crc-32")?)
    }
}

pub struct GzReader<R: Read> {
    member: Member,

    input: GzDecoder<R>,
}

impl<R: Read> Read for GzReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let header = self.input.header().unwrap();
        self.input.read(buf)
    }
}
