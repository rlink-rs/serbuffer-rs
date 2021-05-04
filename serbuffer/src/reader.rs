
use std::io::ErrorKind;
use std::borrow::Borrow;
use std::ops::Deref;

use crate::{Buffer, types};

pub trait Reader<'a, 'b> {

    fn raw_buffer(&self) -> &'a Buffer;

    fn data_types(&self) -> &'b [u8];

    fn index_out_of_bounds_check(
        &self,
        index: usize,
        field_len: usize,
        data_type: u8,
    ) -> Result<(), std::io::Error> {
        if self.raw_buffer().field_pos_index[index] + field_len > self.raw_buffer().buf_len {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }

        if self.data_types()[index] != data_type {
            return Err(std::io::Error::from(ErrorKind::InvalidData));
        }

        Ok(())
    }

     fn get_bool(&self, index: usize) -> Result<bool, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::BOOL)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 1)
            .map(|x| x[0] == 1)
            .unwrap();

        Ok(s)
    }

     fn get_i8(&self, index: usize) -> Result<i8, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::I8)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 1)
            .map(|x| x[0] as i8)
            .unwrap();

        Ok(s)
    }

     fn get_u8(&self, index: usize) -> Result<u8, std::io::Error> {
        self.index_out_of_bounds_check(index, 1, types::U8)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 1)
            .map(|x| x[0])
            .unwrap();

        Ok(s)
    }

     fn get_i16(&self, index: usize) -> Result<i16, std::io::Error> {
        self.index_out_of_bounds_check(index, 2, types::I16)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 2)
            .map(|x| unsafe { i16::from_le_bytes(*(x as *const _ as *const [_; 2])) })
            .unwrap();

        Ok(s)
    }

     fn get_u16(&self, index: usize) -> Result<u16, std::io::Error> {
        self.index_out_of_bounds_check(index, 2, types::U16)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 2)
            .map(|x| unsafe { u16::from_le_bytes(*(x as *const _ as *const [_; 2])) })
            .unwrap();

        Ok(s)
    }

     fn get_i32(&self, index: usize) -> Result<i32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::I32)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { i32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

     fn get_u32(&self, index: usize) -> Result<u32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::U32)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

     fn get_i64(&self, index: usize) -> Result<i64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::I64)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { i64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

     fn get_u64(&self, index: usize) -> Result<u64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::U64)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { u64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

     fn get_f32(&mut self, index: usize) -> Result<f32, std::io::Error> {
        self.index_out_of_bounds_check(index, 4, types::F32)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { f32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        Ok(s)
    }

     fn get_f64(&self, index: usize) -> Result<f64, std::io::Error> {
        self.index_out_of_bounds_check(index, 8, types::F64)?;

        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 8)
            .map(|x| unsafe { f64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
            .unwrap();

        Ok(s)
    }

     fn get_str(&self, index: usize) -> Result<&'a str, std::io::Error> {
        match self.get_bytes(index) {
            Ok(bytes) => {
                std::str::from_utf8(bytes).map_err(|_e|std::io::Error::from(ErrorKind::InvalidData))
            },
            Err(e) => Err(e),
        }
    }

     fn get_bytes(&self, index: usize) -> Result<&'a [u8], std::io::Error> {
        let start = self.raw_buffer().field_pos_index[index];
        let s = self
            .raw_buffer()
            .buf
            .get(start..start + 4)
            .map(|x| unsafe { u32::from_le_bytes(*(x as *const _ as *const [_; 4])) })
            .unwrap();

        let len = s as usize;

        self.index_out_of_bounds_check(index, len + 4, types::BYTES)?;

        let start = start + 4;

        let s = self.raw_buffer().buf.get(start..start + len).unwrap();
        Ok(s)
    }

     fn get_bytes_raw(&self, index: usize) -> Result<&'a [u8], std::io::Error> {
        let data_type = self.data_types()[index];
        if data_type == types::BYTES {
            self.get_bytes(index)
        } else {
            let len = types::len(data_type) as usize;
            let start = self.raw_buffer().field_pos_index[index];

            let s = self.raw_buffer().buf.get(start..start + len).unwrap();

            Ok(s)
        }
    }
}

pub struct BufferReader<'a, 'b> {
    raw_buffer: &'a Buffer,
    data_types: &'b [u8],
}

impl<'a, 'b> BufferReader<'a, 'b> {
    pub(crate) fn new(raw_buffer: &'a Buffer, data_types: &'b [u8]) -> Self {
        BufferReader {
            data_types,
            raw_buffer,
        }
    }
}

impl<'a, 'b> Reader<'a, 'b> for BufferReader<'a, 'b> {
    #[inline]
    fn raw_buffer(&self) -> &'a Buffer {
        self.raw_buffer
    }

    #[inline]
    fn data_types(&self) -> &'b [u8] {
        self.data_types
    }
}

pub struct BufferMutReader<'a, 'b> {
    raw_buffer: &'a mut Buffer,
    data_types: &'b [u8],
}

impl<'a, 'b> BufferMutReader<'a, 'b> {
    pub(crate) fn new(raw_buffer: &'a mut Buffer, data_types: &'b [u8]) -> Self {
        BufferMutReader {
            data_types,
            raw_buffer,
        }
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

impl<'a, 'b> Reader<'a, 'b> for BufferMutReader<'a, 'b> {
    #[inline]
    fn raw_buffer(&self) -> &'a Buffer {
        self.raw_buffer.borrow()
    }

    #[inline]
    fn data_types(&self) -> &'b [u8] {
        self.data_types
    }
}