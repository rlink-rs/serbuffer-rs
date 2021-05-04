//! https://github.com/capnproto/capnproto-rust/blob/master/capnp/src/lib.rs

use bytes::{ BufMut, BytesMut};
use std::hash::Hasher;
use std::io::ErrorKind;

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
    pub const BYTES: u8 = 0b1100_0000;
    pub const STRING: u8 = BYTES;
    // pub const BYTES: u8 = 0b1101_0000;
    // pub const I = 0b1110;
    // pub const I = 0b1111;

    #[inline]
    pub fn len(data_type: u8) -> u8 {
        let length_mod = data_type & 0b0000_1111;
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

    /// field position index cache, build by `Writer` and for `Reader` fast read.
    /// the field is not serialized and deserialized.
    /// must be clear when some create operator such as `new`,`extend`,`reset` ..
    pub(crate) field_pos_index: Vec<usize>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buf: BytesMut::with_capacity(256),
            buf_len: 0,
            field_pos_index: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Buffer {
            buf: BytesMut::with_capacity(capacity),
            buf_len: 0,
            field_pos_index: vec![],
        }
    }

    pub fn from(bytes: BytesMut) -> Self {
        let buffer_len = bytes.len();
        Buffer {
            buf: bytes,
            buf_len: buffer_len,
            field_pos_index: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.buf_len
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_ref()
    }

    pub fn extend(&mut self, other: &Buffer) -> Result<(), std::io::Error> {
        self.field_pos_index.clear();

        self.buf_len += other.buf_len;
        self.buf.put_slice(other.as_slice());

        Ok(())
    }

    pub fn as_reader<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferReader<'a, 'b> {
        self.position_index_cache_check(data_types);

        BufferReader::new(self, data_types)
    }

    pub fn as_reader_mut<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferMutReader<'a, 'b> {
        self.position_index_cache_check(data_types);

        BufferMutReader::new(self, data_types)
    }

    pub fn as_writer<'a, 'b>(&'a mut self, data_types: &'b [u8]) -> BufferWriter<'a, 'b> {
        self.position_index_cache_check(data_types);

        BufferWriter::new(self, data_types)
    }

    fn position_index_cache_check(&mut self, data_types: &[u8]) {
        if self.field_pos_index.len() == 0 && self.buf_len > 0 {
            let mut field_start_pos = 0;
            for index in 0..data_types.len() {
                if field_start_pos > self.buf_len {
                    panic!("read error");
                }

                self.field_pos_index.push(field_start_pos);
                let data_type = data_types[index];
                if data_type == types::BYTES {
                    let len = self
                        .buf
                        .get(field_start_pos..field_start_pos + 4)
                        .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
                        .unwrap();
                    field_start_pos += (len + 4) as usize;
                } else {
                    let len = types::len(data_type);
                    field_start_pos += len as usize;
                }
            }

            if field_start_pos > self.buf_len {
                panic!("read error");
            }
        }
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

pub struct BufferMutReader<'a, 'b> {
    raw_buffer: &'a mut Buffer,
    data_types: &'b [u8],
}

impl<'a, 'b> BufferMutReader<'a, 'b> {
    fn new(raw_buffer: &'a mut Buffer, data_types: &'b [u8]) -> Self {
        BufferMutReader {
            data_types,
            raw_buffer,
        }
    }

    #[inline]
    fn index_out_of_bounds_check(
        &self,
        index: usize,
        field_len: usize,
        data_type: u8,
    ) -> Result<(), std::io::Error> {
        if self.raw_buffer.field_pos_index[index] + field_len > self.raw_buffer.buf_len {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }

        if self.data_types[index] != data_type {
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }

        Ok(())
    }

    pub fn get_bytes_mut(&mut self, index: usize) -> Result<&mut [u8], std::io::Error> {
        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        let len = s as usize;

        self.index_out_of_bounds_check(index, len + 4, types::BYTES)?;

        let start = start + 4;

        let s = self.raw_buffer.buf.get_mut(start..start + len).unwrap();
        Ok(s)
    }

    pub fn get_bytes_raw_mut(&mut self, index: usize) -> Result<&mut [u8], std::io::Error> {
        let data_type = self.data_types[index];
        if data_type == types::BYTES {
            self.get_bytes_mut(index)
        } else {
            let len = types::len(data_type) as usize;
            let start = self.raw_buffer.field_pos_index[index];

            let s = self.raw_buffer.buf.get_mut(start..start + len).unwrap();

            Ok(s)
        }
    }
}

pub struct BufferReader<'a, 'b> {
    raw_buffer: &'a Buffer,
    data_types: &'b [u8],
}

impl<'a, 'b> BufferReader<'a, 'b> {
    fn new(raw_buffer: &'a Buffer, data_types: &'b [u8]) -> Self {
        BufferReader {
            data_types,
            raw_buffer,
        }
    }

    #[inline]
    fn index_out_of_bounds_check(
        &self,
        index: usize,
        field_len: usize,
        data_type: u8,
    ) -> Result<(), std::io::Error> {
        if self.raw_buffer.field_pos_index[index] + field_len > self.raw_buffer.buf_len {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }

        if self.data_types[index] != data_type {
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }

        Ok(())
    }

    pub fn get_bool(&self, index: usize) -> Result<bool, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::BOOL)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 1)
            .map(|x| x[0] == 1)
            .unwrap();

        Ok(s)
    }

    pub fn get_i8(&self, index: usize) -> Result<i8, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::I8)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 1)
            .map(|x| x[0] as i8)
            .unwrap();

        Ok(s)
    }

    pub fn get_u8(&self, index: usize) -> Result<u8, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::U8)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 1)
            .map(|x| x[0])
            .unwrap();

        Ok(s)
    }

    pub fn get_i16(&self, index: usize) -> Result<i16, std::io::Error> {
        self.index_out_of_bounds_check(index, 2, types::I16)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 2)
            .map(|x| unsafe { i16::from_le_bytes(*(x as *const _ as *const [_; 2])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_u16(&self, index: usize) -> Result<u16, std::io::Error> {
        self.index_out_of_bounds_check(index, 2, types::U16)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 2)
            .map(|x| unsafe { u16::from_le_bytes(*(x as *const _ as *const [_; 2])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_i32(&self, index: usize) -> Result<i32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::I32)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { i32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_u32(&self, index: usize) -> Result<u32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::U32)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_i64(&self, index: usize) -> Result<i64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::I64)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { i64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_u64(&self, index: usize) -> Result<u64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::U64)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { u64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_f32(&mut self, index: usize) -> Result<f32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::F32)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { f32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_f64(&self, index: usize) -> Result<f64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::F64)?;

        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { f64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

    pub fn get_str(&self, index: usize) -> Result<&'a str, std::io::Error> {
        match self.get_bytes(index) {
            Ok(bytes) => {
                std::str::from_utf8(bytes).map_err(|_e|std::io::Error::from(ErrorKind::InvalidData))
            },
            Err(e) => Err(e),
        }
    }

    pub fn get_bytes(&self, index: usize) -> Result<&'a [u8], std::io::Error> {
        let start = self.raw_buffer.field_pos_index[index];
        let s = self
            .raw_buffer
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        let len = s as usize;

        self.index_out_of_bounds_check(index, len + 4, types::BYTES)?;

        let start = start + 4;

        let s = self.raw_buffer.buf.get(start..start + len).unwrap();
        Ok(s)
    }

    pub fn get_bytes_raw(&self, index: usize) -> Result<&[u8], std::io::Error> {
        let data_type = self.data_types[index];
        if data_type == types::BYTES {
            self.get_bytes(index)
        } else {
            let len = types::len(data_type) as usize;
            let start = self.raw_buffer.field_pos_index[index];

            let s = self.raw_buffer.buf.get(start..start + len).unwrap();

            Ok(s)
        }
    }

}

pub struct BufferWriter<'a, 'b> {
    raw_buffer: &'a mut Buffer,
    data_types: &'b [u8],
    write_field_step: usize,
    // write_position: usize,
}

impl<'a, 'b> BufferWriter<'a, 'b> {
    fn new(raw_buffer: &'a mut Buffer, data_types: &'b [u8]) -> Self {
        BufferWriter {
            raw_buffer,
            data_types,
            write_field_step: 0,
        }
    }

    #[inline]
    fn data_type_check(&mut self, data_type: u8) -> Result<(), std::io::Error> {
        if self.data_types[self.write_field_step] != data_type {
            return Err(std::io::Error::from(ErrorKind::InvalidInput));
        }

        self.write_field_step += 1;
        Ok(())
    }

    #[inline]
    fn step_position(&mut self, pos_step_len: usize) {
        self.raw_buffer
            .field_pos_index
            .push(self.raw_buffer.buf_len);
        self.raw_buffer.buf_len += pos_step_len;
    }

    pub fn set_bool(&mut self, value: bool) -> Result<(), std::io::Error> {
        self.data_type_check(types::BOOL)?;

        let value = if value { 1 } else { 0 };

        self.step_position(1);

        self.raw_buffer.buf.put_u8(value);
        Ok(())
    }

    pub fn set_i8(&mut self, value: i8) -> Result<(), std::io::Error> {
        self.data_type_check(types::I8)?;

        self.step_position(1);

        self.raw_buffer.buf.put_i8(value);
        Ok(())
    }

    pub fn set_u8(&mut self, value: u8) -> Result<(), std::io::Error> {
        self.data_type_check(types::U8)?;

        self.step_position(1);

        self.raw_buffer.buf.put_u8(value);
        Ok(())
    }

    pub fn set_i16(&mut self, value: i16) -> Result<(), std::io::Error> {
        self.data_type_check(types::I16)?;

        self.step_position(2);

        self.raw_buffer.buf.put_i16_le(value);
        Ok(())
    }

    pub fn set_u16(&mut self, value: u16) -> Result<(), std::io::Error> {
        self.data_type_check(types::U16)?;

        self.step_position(2);

        self.raw_buffer.buf.put_u16_le(value);
        Ok(())
    }

    pub fn set_i32(&mut self, value: i32) -> Result<(), std::io::Error> {
        self.data_type_check(types::I32)?;

        self.step_position(4);

        self.raw_buffer.buf.put_i32_le(value);
        Ok(())
    }

    pub fn set_u32(&mut self, value: u32) -> Result<(), std::io::Error> {
        self.data_type_check(types::U32)?;

        self.step_position(4);

        self.raw_buffer.buf.put_u32_le(value);
        Ok(())
    }

    pub fn set_i64(&mut self, value: i64) -> Result<(), std::io::Error> {
        self.data_type_check(types::I64)?;

        self.step_position(8);

        self.raw_buffer.buf.put_i64_le(value);
        Ok(())
    }

    pub fn set_u64(&mut self, value: u64) -> Result<(), std::io::Error> {
        self.data_type_check(types::U64)?;

        self.step_position(8);

        self.raw_buffer.buf.put_u64_le(value);
        Ok(())
    }

    pub fn set_f32(&mut self, value: f32) -> Result<(), std::io::Error> {
        self.data_type_check(types::F32)?;

        self.step_position(4);

        self.raw_buffer.buf.put_f32_le(value);
        Ok(())
    }

    pub fn set_f64(&mut self, value: f64) -> Result<(), std::io::Error> {
        self.data_type_check(types::F64)?;

        self.step_position(8);

        self.raw_buffer.buf.put_f64_le(value);
        Ok(())
    }

    pub fn set_str(&mut self, value: &str) -> Result<(), std::io::Error> {
        let s = value.as_bytes();
        self.set_bytes(s)
    }

    pub fn set_bytes(&mut self, value: &[u8]) -> Result<(), std::io::Error> {
        self.data_type_check(types::BYTES)?;

        let len = value.len();
        self.step_position(len + 4);

        self.raw_buffer.buf.put_u32_le(len as u32);
        self.raw_buffer.buf.put_slice(value);
        Ok(())
    }

    pub fn set_bytes_raw(&mut self, value: &[u8]) -> Result<(), std::io::Error> {
        let data_type = self.data_types[self.write_field_step];
        if data_type == types::BYTES {
            self.set_bytes(value)
        } else {
            let len = types::len(data_type) as usize;
            if len != value.len() {
                return Err(std::io::Error::from(ErrorKind::InvalidInput));
            }

            self.write_field_step += 1;

            self.step_position(len);

            self.raw_buffer.buf.put_slice(value);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{types, Buffer};

    #[test]
    pub fn buffer_test() {
        let mut buffer = Buffer::new();
        let data_types = vec![
            types::BOOL,
            types::I8,
            types::U8,
            types::I16,
            types::U16,
            types::I32,
            types::U32,
            types::I64,
            types::U64,
            types::F32,
            types::F64,
            types::BYTES,
            types::BYTES,
            types::BYTES,
            types::I32,
        ];

        let strs = vec![
            "aaaa-bbbb-cccc-dddd",
            "cccc-bbbb-aaaa",
            "aaaa-bbbb-cccc-dddd-eeee",
            "dddd-cccc-bbbb-aaaa",
            "bbbb",
        ];
        for i in 0..strs.len() {
            let uuid1 = strs[i];
            let uuid2 = uuid::Uuid::new_v4().to_string();
            let uuid2 = uuid2.as_str();

            let mut writer = buffer.as_writer(&data_types);

            writer.set_bool(i % 2 == 0).unwrap();

            writer.set_i8((10 + i) as i8).unwrap();
            writer.set_u8((11 + i) as u8).unwrap();

            writer.set_i16((12 + i) as i16).unwrap();
            writer.set_u16((13 + i) as u16).unwrap();

            writer.set_i32((14 + i) as i32).unwrap();
            writer.set_u32((15 + i) as u32).unwrap();

            writer.set_i64((16 + i) as i64).unwrap();
            writer.set_u64((17 + i) as u64).unwrap();

            writer.set_f32(18.001 + i as f32).unwrap();
            writer.set_f64(19.002 + i as f64).unwrap();

            writer.set_str(uuid1).unwrap();
            writer.set_str(uuid2).unwrap();
            writer.set_str("").unwrap();

            writer.set_i32((5 + i) as i32).unwrap();

            println!("{:?}", buffer.buf.as_ref());

            let mut reader = buffer.as_reader(&data_types);

            assert_eq!(reader.get_bool(0).unwrap(), i % 2 == 0);

            assert_eq!(reader.get_i8(1).unwrap(), (10 + i) as i8);
            assert_eq!(reader.get_u8(2).unwrap(), (11 + i) as u8);

            assert_eq!(reader.get_i16(3).unwrap(), (12 + i) as i16);
            assert_eq!(reader.get_u16(4).unwrap(), (13 + i) as u16);

            assert_eq!(reader.get_i32(5).unwrap(), (14 + i) as i32);
            assert_eq!(reader.get_u32(6).unwrap(), (15 + i) as u32);

            assert_eq!(reader.get_i64(7).unwrap(), (16 + i) as i64);
            assert_eq!(reader.get_u64(8).unwrap(), (17 + i) as u64);

            assert_eq!(reader.get_f32(9).unwrap(), 18.001 + i as f32);
            assert_eq!(reader.get_f64(10).unwrap(), 19.002 + i as f64);

            assert_eq!(reader.get_str(11).unwrap(), uuid1.to_string());
            assert_eq!(reader.get_str(12).unwrap(), uuid2.to_string());
            assert_eq!(reader.get_str(13).unwrap(), "".to_string());

            assert_eq!(reader.get_i32(14).unwrap(), (5 + i) as i32);

            // buffer.reset();
        }
    }

    #[test]
    pub fn buf_extend_test() {
        let mut buffer0 = Buffer::new();
        let mut buffer1 = Buffer::new();

        let data_types0 = [
            types::I8,
            types::I16,
            types::I32,
            types::I64,
            types::F32,
            types::BYTES,
            types::I32,
        ];
        let data_types1 = [
            types::U8,
            types::U16,
            types::U32,
            types::U64,
            types::F64,
            types::BYTES,
            types::U32,
        ];

        let uuid1 = "b871544b-c044-495c-98ee-f5aa34660527".to_string();
        let uuid2 = "6af8fda0-e9fa-498b-829a-bb3c7b87554b".to_string();

        {
            let mut writer = buffer0.as_writer(&data_types0);

            writer.set_i8((10) as i8).unwrap();
            writer.set_i16((12) as i16).unwrap();
            writer.set_i32((14) as i32).unwrap();
            writer.set_i64((16) as i64).unwrap();
            writer.set_f32(18.001 as f32).unwrap();
            writer.set_str(uuid1.as_str()).unwrap();
            writer.set_i32((5) as i32).unwrap();
        }

        {
            let mut writer = buffer1.as_writer(&data_types1);

            writer.set_u8((11) as u8).unwrap();
            writer.set_u16((13) as u16).unwrap();
            writer.set_u32((15) as u32).unwrap();
            writer.set_u64((17) as u64).unwrap();
            writer.set_f64(19.002 as f64).unwrap();
            writer.set_str(uuid2.as_str()).unwrap();
            writer.set_u32((5) as u32).unwrap();
        }

        println!("{:?}", buffer0.buf.as_ref());
        println!("{:?}", buffer1.buf.as_ref());

        buffer0.extend(&buffer1).unwrap();
        println!("{:?}", buffer0.buf.as_ref());

        let mut data_type_merge = data_types0.to_vec();
        data_type_merge.extend_from_slice(&data_types1);
        let mut reader = buffer0.as_reader(data_type_merge.as_slice());

        assert_eq!(reader.get_i8(0).unwrap(), (10) as i8);
        assert_eq!(reader.get_i16(1).unwrap(), (12) as i16);
        assert_eq!(reader.get_i32(2).unwrap(), (14) as i32);
        assert_eq!(reader.get_i64(3).unwrap(), (16) as i64);
        assert_eq!(reader.get_f32(4).unwrap(), 18.001 as f32);
        assert_eq!(reader.get_str(5).unwrap(), uuid1);
        assert_eq!(reader.get_i32(6).unwrap(), (5) as i32);

        assert_eq!(reader.get_u8(7).unwrap(), (11) as u8);
        assert_eq!(reader.get_u16(8).unwrap(), (13) as u16);
        assert_eq!(reader.get_u32(9).unwrap(), (15) as u32);
        assert_eq!(reader.get_u64(10).unwrap(), (17) as u64);
        assert_eq!(reader.get_f64(11).unwrap(), 19.002 as f64);
        assert_eq!(reader.get_str(12).unwrap(), uuid2);
        assert_eq!(reader.get_u32(13).unwrap(), (5) as u32);
    }
}
