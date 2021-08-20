use std::borrow::BorrowMut;
use std::io::ErrorKind;

use bytes::BufMut;

use crate::encoding::write_lenenc_int;
use crate::{types, Buffer};

pub struct BufferWriter<'a, 'b> {
    raw_buffer: &'a mut Buffer,
    data_types: &'b [u8],
    write_field_step: usize,
}

impl<'a, 'b> BufferWriter<'a, 'b> {
    pub(crate) fn new(raw_buffer: &'a mut Buffer, data_types: &'b [u8]) -> Self {
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
        self.set_bytes(s, types::STRING)
    }

    pub fn set_binary(&mut self, value: &[u8]) -> Result<(), std::io::Error> {
        self.set_bytes(value, types::BINARY)
    }

    fn set_bytes(&mut self, value: &[u8], data_type_id: u8) -> Result<(), std::io::Error> {
        self.data_type_check(data_type_id)?;

        let len = value.len();

        let len_length = write_lenenc_int(len as u64, self.raw_buffer.buf.borrow_mut());
        self.raw_buffer.buf.put_slice(value);

        self.step_position(len + len_length);

        Ok(())
    }

    pub fn set_bytes_raw(&mut self, value: &[u8]) -> Result<(), std::io::Error> {
        let data_type_id = self.data_types[self.write_field_step];
        if data_type_id >= types::BINARY {
            self.set_bytes(value, data_type_id)
        } else {
            let len = types::len(data_type_id) as usize;
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
