#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![rustfmt::skip]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file by schema GenDemo

use serbuffer::{types, BufferReader, BufferWriter, Buffer};

pub mod index {
    pub const timestamp: usize = 0;
    pub const index: usize = 1;
    pub const group: usize = 2;
    pub const service: usize = 3;
    pub const count: usize = 4;
}

pub const FIELD_TYPE: [u8; 5] = [
    // 0: timestamp
    types::U64,
    // 1: index
    types::U8,
    // 2: group
    types::STRING,
    // 3: service
    types::I32,
    // 4: count
    types::I64,
];

pub struct FieldReader<'a> {
    reader: BufferReader<'a, 'static>,
}

impl<'a> FieldReader<'a> {
    pub fn new(b: &'a mut Buffer) -> Self {
        let reader = b.as_reader(&FIELD_TYPE);
        FieldReader { reader }
    }

    pub fn get_timestamp(&mut self) -> Result<u64, std::io::Error> {
        self.reader.get_u64(0)
    }

    pub fn get_index(&mut self) -> Result<u8, std::io::Error> {
        self.reader.get_u8(1)
    }

    pub fn get_group(&mut self) -> Result<&str, std::io::Error> {
        self.reader.get_str(2)
    }

    pub fn get_service(&mut self) -> Result<i32, std::io::Error> {
        self.reader.get_i32(3)
    }

    pub fn get_count(&mut self) -> Result<i64, std::io::Error> {
        self.reader.get_i64(4)
    }
}

pub struct FieldWriter<'a> {
    writer: BufferWriter<'a, 'static>,
    writer_pos: usize,
}

impl<'a> FieldWriter<'a> {
    pub fn new(b: &'a mut Buffer) -> Self {
        let writer = b.as_writer(&FIELD_TYPE);
        FieldWriter {
            writer,
            writer_pos: 0,
        }
    }

    pub fn set_timestamp(&mut self, timestamp: u64) -> Result<(), std::io::Error> {
        if self.writer_pos == 0 {
            self.writer_pos += 1;
            self.writer.set_u64(timestamp)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`timestamp` must be set sequentially"))
        }
    }

    pub fn set_index(&mut self, index: u8) -> Result<(), std::io::Error> {
        if self.writer_pos == 1 {
            self.writer_pos += 1;
            self.writer.set_u8(index)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`index` must be set sequentially"))
        }
    }

    pub fn set_group(&mut self, group: &str) -> Result<(), std::io::Error> {
        if self.writer_pos == 2 {
            self.writer_pos += 1;
            self.writer.set_str(group)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`group` must be set sequentially"))
        }
    }

    pub fn set_service(&mut self, service: i32) -> Result<(), std::io::Error> {
        if self.writer_pos == 3 {
            self.writer_pos += 1;
            self.writer.set_i32(service)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`service` must be set sequentially"))
        }
    }

    pub fn set_count(&mut self, count: i64) -> Result<(), std::io::Error> {
        if self.writer_pos == 4 {
            self.writer_pos += 1;
            self.writer.set_i64(count)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`count` must be set sequentially"))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entity<'a> {
    pub timestamp: u64,
    pub index: u8,
    pub group: &'a str,
    pub service: i32,
    pub count: i64,
}

impl<'a> Entity<'a> {
    pub fn to_buffer(&self, b: &mut Buffer) -> Result<(), std::io::Error> {
        let mut writer = b.as_writer(&FIELD_TYPE);
        
        writer.set_u64(self.timestamp)?;
        writer.set_u8(self.index)?;
        writer.set_str(self.group)?;
        writer.set_i32(self.service)?;
        writer.set_i64(self.count)?;

        Ok(())
    }
    
    pub fn parse(b: &'a mut Buffer) -> Result<Self, std::io::Error> {
        let reader = b.as_reader(&FIELD_TYPE);

        let entity = Entity {
            timestamp: reader.get_u64(0)?,
            index: reader.get_u8(1)?,
            group: reader.get_str(2)?,
            service: reader.get_i32(3)?,
            count: reader.get_i64(4)?,
        };

        Ok(entity)
    }
}
