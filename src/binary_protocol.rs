use protocol::{Serializer, Deserializer, ThriftSerializer, ThriftField, ThriftMessage, ThriftDeserializer, ThriftMessageType, ThriftType, Error};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::iter;
use byteorder;
use std::convert;

pub const THRIFT_VERSION_1: i32 = 0x80010000;
pub const THRIFT_VERSION_MASK: i32 = 0xffff0000;
pub const THRIFT_TYPE_MASK: i32 = 0x000000ff;

pub struct BinarySerializer<'a> {
    wr: &'a mut Write
}

impl<'a> BinarySerializer<'a> {
    pub fn new(wr: &'a mut Write) -> BinarySerializer<'a> {
        BinarySerializer {
            wr: wr
        }
    }
}

impl<'a> Serializer for BinarySerializer<'a> {

    fn serialize_bool(&mut self, val: bool) -> Result<(), Error> {
        if val {
            self.serialize_i8(1)
        } else {
            self.serialize_i8(0)
        }
    }

    fn serialize_usize(&mut self, val: usize) -> Result<(), Error> {
        self.serialize_isize(val as isize)
    }

    fn serialize_isize(&mut self, val: isize) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_u64(&mut self, val: u64) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(self.wr.write_i64::<BigEndian>(val));
        Ok(())
    }

    fn serialize_u32(&mut self, val: u32) -> Result<(), Error> {
        self.serialize_i32(val as i32)
    }

    fn serialize_i32(&mut self, val: i32) -> Result<(), Error> {
        try!(self.wr.write_i32::<BigEndian>(val));
        Ok(())
    }

    fn serialize_u16(&mut self, val: u16) -> Result<(), Error> {
        self.serialize_i16(val as i16)
    }

    fn serialize_i16(&mut self, val: i16) -> Result<(), Error> {
        try!(self.wr.write_i16::<BigEndian>(val));
        Ok(())
    }

    fn serialize_u8(&mut self, val: u8) -> Result<(), Error> {
        self.serialize_i8(val as i8)
    }

    fn serialize_i8(&mut self, val: i8) -> Result<(), Error> {
        try!(self.wr.write_i8(val));
        Ok(())
    }

    fn serialize_bytes(&mut self, val: &[u8]) -> Result<(), Error> {
        self.serialize_i32(val.len() as i32);
        try!(self.wr.write(val));
        Ok(())
    }

    fn serialize_str(&mut self, val: &str) -> Result<(), Error> {
        self.serialize_bytes(val.as_bytes())
    }

    fn serialize_string(&mut self, val: String) -> Result<(), Error> {
        self.serialize_str(&*val)
    }
}

impl<'a> ThriftSerializer for BinarySerializer<'a> {
    fn write_message_begin(&mut self, name: &str, message_type: ThriftMessageType) -> Result<(), Error> {
        let version = THRIFT_VERSION_1 | message_type as i32;

        try!(self.serialize_i32(version));
        try!(self.serialize_str(name));
        try!(self.serialize_i16(0));

        Ok(())
    }

    fn write_struct_begin(&mut self, name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn write_struct_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_field_begin(&mut self, name: &str, ty: ThriftType, id: i16) -> Result<(), Error> {
        try!(self.serialize_i8(ty as i8));
        try!(self.serialize_i16(id));
        Ok(())
    }

    fn write_field_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_field_stop(&mut self) -> Result<(), Error> {
        try!(self.serialize_i8(ThriftType::Stop as i8));
        Ok(())
    }

    fn write_message_end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct BinaryDeserializer<R: Read + ReadBytesExt> {
    rd: R
}

impl<R: Read + ReadBytesExt> BinaryDeserializer<R> {
    pub fn new(rd: R) -> BinaryDeserializer<R> {
        BinaryDeserializer {
            rd: rd
        }
    }
}

impl<R: Read + ReadBytesExt> Deserializer for BinaryDeserializer<R> {
    fn deserialize_bool(&mut self) -> Result<bool, Error> {
        Ok(try!(self.rd.read_i8()) != 0)
    }

    fn deserialize_usize(&mut self) -> Result<usize, Error> {
        Ok(try!(self.deserialize_isize()) as usize)
    }

    fn deserialize_isize(&mut self) -> Result<isize, Error> {
        Ok(try!(self.deserialize_i64()) as isize)
    }

    fn deserialize_u64(&mut self) -> Result<u64, Error> {
        Ok(try!(self.deserialize_i64()) as u64)
    }

    fn deserialize_i64(&mut self) -> Result<i64, Error> {
        Ok(try!(self.rd.read_i64::<BigEndian>()))
    }

    fn deserialize_u32(&mut self) -> Result<u32, Error> {
        Ok(try!(self.deserialize_i32()) as u32)
    }

    fn deserialize_i32(&mut self) -> Result<i32, Error> {
        Ok(try!(self.rd.read_i32::<BigEndian>()))
    }

    fn deserialize_u16(&mut self) -> Result<u16, Error> {
        Ok(try!(self.deserialize_i16()) as u16)
    }

    fn deserialize_i16(&mut self) -> Result<i16, Error> {
        Ok(try!(self.rd.read_i16::<BigEndian>()))
    }

    fn deserialize_u8(&mut self) -> Result<u8, Error> {
        Ok(try!(self.deserialize_i8()) as u8)
    }

    fn deserialize_i8(&mut self) -> Result<i8, Error> {
        Ok(try!(self.rd.read_i8()))
    }

    fn deserialize_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = try!(self.deserialize_i32()) as usize;
        let mut buf = Vec::with_capacity(len);

        buf.extend(iter::repeat(0).take(len));
        try!(self.rd.read(&mut buf));

        Ok(buf)
    }

    fn deserialize_str(&mut self) -> Result<String, Error> {
        let buf = try!(self.deserialize_bytes());
        let s = try!(String::from_utf8(buf));
        Ok(s)
    }
}

impl<R: Read + ReadBytesExt> ThriftDeserializer for BinaryDeserializer<R> {
    fn read_message_begin(&mut self) -> Result<ThriftMessage, Error> {
        let size: i32 = try!(self.deserialize_i32());

        if size < 0 {
            let version = size & THRIFT_VERSION_MASK;
            if version != THRIFT_VERSION_1 {
                Err(Error::BadVersion)
            } else {
                Ok(ThriftMessage {
                    name: try!(self.deserialize_str()),
                    ty: ThriftMessageType::from((size & THRIFT_TYPE_MASK) as i8),
                    seq: try!(self.deserialize_i16())
                })
            }
        } else {
            Err(Error::ProtocolVersionMissing)
        }
    }

    fn read_message_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn read_struct_begin(&mut self) -> Result<String, Error> {
        Ok("".to_string())
    }

    fn read_struct_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn read_field_begin(&mut self) -> Result<ThriftField, Error> {
        let mut field = ThriftField {
            name: None,
            ty: ThriftType::from(try!(self.deserialize_i8())),
            seq: 0
        };

        if field.ty == ThriftType::Stop {
            Ok(field)
        } else {
            field.seq = try!(self.deserialize_i16());
            Ok(field)
        }
    }

    fn read_field_end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use byteorder::{ReadBytesExt, BigEndian};
    use protocol::{ThriftMessageType, ThriftType, ThriftSerializer, Serializer, Serialize, Deserializer};
    use super::*;

    #[test]
    fn serialize_bool_true() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_bool(true);
        }

        assert_eq!(v[0], 1);
    }

    #[test]
    fn serialize_bool_false() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_bool(false);
        }

        assert_eq!(v[0], 0);
    }

    #[test]
    fn serialize_i8() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i8(5);
        }

        assert_eq!(v[0], 5);
    }

    #[test]
    fn serialize_i8_neg() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i8(-5);
        }

        assert_eq!(v[0] as i8, -5);
    }

    #[test]
    fn serialize_i16() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i16(900);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(900, cursor.read_i16::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i16_neg() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i16(-900);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-900, cursor.read_i16::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i32() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i32(3000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(3000000, cursor.read_i32::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i32_neg() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i32(-3000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-3000000, cursor.read_i32::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i64() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i64(33000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(33000000, cursor.read_i64::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i64_neg() {
        let mut v = Vec::new();
        {
            let mut s = BinarySerializer::new(&mut v);
            s.serialize_i64(-33000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-33000000, cursor.read_i64::<BigEndian>().unwrap());
    }

    #[test]
    fn protocol_begin() {
        let mut v = Vec::new();
        {
            let mut proto = BinarySerializer::new(&mut v);
            proto.write_message_begin("foobar", ThriftMessageType::Call);
        }

        let mut cursor = Cursor::new(v);
        let version = THRIFT_VERSION_1 | ThriftMessageType::Call as i32;

        assert_eq!(version, cursor.read_i32::<BigEndian>().unwrap());
        // XXX Decode string and seqid.
    }
}
