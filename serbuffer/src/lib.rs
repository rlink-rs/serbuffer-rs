pub mod buffer;
pub mod reader;
pub mod writer;

pub use buffer::types;
pub use buffer::Buffer;
pub use reader::BufferMutReader;
pub use reader::BufferReader;
pub use writer::BufferWriter;

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
