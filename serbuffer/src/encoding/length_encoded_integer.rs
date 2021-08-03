//! reference: https://dev.mysql.com/doc/internals/en/integer.html#length-encoded-integer

use std::io;

use bytes::{BufMut, BytesMut};

pub fn write_lenenc_int(x: u64, buf: &mut BytesMut) -> usize {
    if x < 251 {
        buf.put_u8(x as u8);
        1
    } else if x < 65_536 {
        buf.put_u8(0xFC);
        buf.put_u16_le(x as u16);
        3
    } else if x < 16_777_216 {
        buf.put_u8(0xFD);
        let n: [u8; 4] = (x as u32).to_le_bytes();
        buf.put(&n[0..3]);
        4
    } else {
        buf.put_u8(0xFE);
        buf.put_u64_le(x);
        9
    }
}

pub fn read_lenenc_int(buf: &BytesMut, offset: usize) -> io::Result<(u64, usize)> {
    let flag = &buf[offset];
    match *flag {
        x if x < 0xFC => Ok((x as u64, 1)),
        0xFC => {
            let v = buf
                .get(offset + 1..offset + 3)
                .map(|x| unsafe { u16::from_le_bytes(*(x as *const _ as *const [_; 2])) })
                .unwrap();
            Ok((v as u64, 3))
        }
        0xFD => {
            let b = buf.get(offset + 1..offset + 4).unwrap();
            let le_4_bytes = [b[0], b[1], b[2], 0u8];
            Ok((u32::from_le_bytes(le_4_bytes) as u64, 4))
        }
        0xFE => {
            let v = buf
                .get(offset + 1..offset + 9)
                .map(|x| unsafe { u64::from_le_bytes(*(x as *const _ as *const [_; 8])) })
                .unwrap();
            Ok((v, 9))
        }
        0xFF => Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid length-encoded integer value",
        )),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use std::borrow::BorrowMut;

    use crate::encoding::length_encoded_integer::{read_lenenc_int, write_lenenc_int};

    #[test]
    pub fn lenenc_int_0_test() {
        let mut bs = BytesMut::new();

        ////////////////////////////////////////////////////////////////////////////////////////////
        // write
        ////////////////////////////////////////////////////////////////////////////////////////////

        let len = write_lenenc_int(0, bs.borrow_mut());
        assert_eq!(len, 1);
        let len = write_lenenc_int(251 - 1, bs.borrow_mut());
        assert_eq!(len, 1);

        let len = write_lenenc_int(251, bs.borrow_mut());
        assert_eq!(len, 3);
        let len = write_lenenc_int(65_536 - 1, bs.borrow_mut());
        assert_eq!(len, 3);

        let len = write_lenenc_int(65_536, bs.borrow_mut());
        assert_eq!(len, 4);
        let len = write_lenenc_int(16_777_216 - 1, bs.borrow_mut());
        assert_eq!(len, 4);

        let len = write_lenenc_int(16_777_216, bs.borrow_mut());
        assert_eq!(len, 9);

        ////////////////////////////////////////////////////////////////////////////////////////////
        // read
        ////////////////////////////////////////////////////////////////////////////////////////////

        let mut offset = 0;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 0);
        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 251 - 1);

        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 251);
        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 65_536 - 1);

        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 65_536);
        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 16_777_216 - 1);

        offset += v_read.1;
        let v_read = read_lenenc_int(&bs, offset).unwrap();
        assert_eq!(v_read.0, 16_777_216);
    }
}
