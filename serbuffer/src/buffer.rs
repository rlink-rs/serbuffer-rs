//! https://github.com/capnproto/capnproto-rust/blob/master/capnp/src/lib.rs

use bytes::{BufMut, BytesMut};

use crate::reader::{BufferMutReader, BufferReader};
use crate::writer::BufferWriter;

pub mod types {
    /// types: 0b[type]_[length_mod]
    /// length = if length_mod == 0 then 0 else 2 << (length_mod -1) .
    ///     eg: BOOL,I8,U8 = 0
    ///         I16,U16 = 2 << (1-1) = 2
    ///         I32,U32,F32 = 2 << (2-1) = 4
    ///         I64,U64,F64 = 2 << (3-1) = 8
    pub const BOOL: u8 = 0b0000_0000;
    pub const I8: u8 = 0b0001_0000;
    pub const U8: u8 = 0b0010_0000;
    pub const I16: u8 = 0b0011_0001;
    pub const U16: u8 = 0b0100_0001;
    pub const I32: u8 = 0b0101_0010;
    pub const U32: u8 = 0b0111_0010;
    pub const I64: u8 = 0b1000_0011;
    pub const U64: u8 = 0b1001_0011;
    pub const F32: u8 = 0b1010_0010;
    pub const F64: u8 = 0b1011_0011;

    pub const BINARY: u8 = 0b1100_0000;
    pub const STRING: u8 = 0b1100_0001;

    #[inline]
    pub fn len(data_type_id: u8) -> u8 {
        let length_mod = data_type_id & 0b0000_1111;
        if length_mod == 0 {
            1
        } else {
            2 << (length_mod - 1)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub(crate) buf: BytesMut,
    pub(crate) buf_len: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buf: BytesMut::with_capacity(256),
            buf_len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Buffer {
            buf: BytesMut::with_capacity(capacity),
            buf_len: 0,
        }
    }

    pub fn from(bytes: BytesMut) -> Self {
        let buffer_len = bytes.len();
        Buffer {
            buf: bytes,
            buf_len: buffer_len,
        }
    }

    pub fn len(&self) -> usize {
        self.buf_len
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_ref()
    }

    pub fn extend(&mut self, other: &Buffer) -> Result<(), std::io::Error> {
        self.buf_len += other.buf_len;
        self.buf.put_slice(other.as_slice());

        Ok(())
    }

    pub fn as_reader<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferReader<'a, 'b> {
        BufferReader::new(self, data_types)
    }

    pub fn as_reader_mut<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferMutReader<'a, 'b> {
        BufferMutReader::new(self, data_types)
    }

    pub fn as_writer<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferWriter<'a, 'b> {
        BufferWriter::new(self, data_types)
    }
}

impl std::cmp::PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        if self.buf_len != self.buf_len {
            return false;
        }

        self.as_slice().eq(other.as_slice())
    }
}

impl std::cmp::Eq for Buffer {}

impl std::hash::Hash for Buffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
