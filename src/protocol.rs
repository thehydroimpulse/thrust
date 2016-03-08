use std::io::{Write, Read};
use std::fmt::{self, Display};
use std::convert::Into;
use std::error;
use serde::{Serialize, Serializer as Ser};
use serde;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub enum ThriftType {
    Stop = 0,
    Void = 1,
    Bool = 2,
    Byte = 3,
    I16 = 6,
    I32 = 8,
    U64 = 9,
    I64 = 10,
    Double = 4,
    String = 11,
    Struct = 12,
    Map = 13,
    Set = 14,
    List = 15
}

pub enum ThriftMessageType {
    Call = 1,
    Reply = 2,
    Exception = 3,
    Oneway = 4
}

pub const THRIFT_VERSION_1: i32 = 0x80010000;
pub const THRIFT_VERSION_MASK: i32 = 0xffff0000;
pub const THRIFT_TYPE_MASK: i32 = 0x000000ff;

pub trait Protocol {
    fn write_message_begin(&mut self, name: &str, message_type: ThriftMessageType);
    // fn write_message_end(&mut self);
    // fn write_struct_begin(&mut self);
    // fn write_struct_end(&mut self);
    // fn write_field_begin(&mut self, ty: u8, id: i16);
    // fn write_field_end(&mut self);
    // fn write_field_stop(&mut self);
    // fn write_bool(&mut self, val: bool);
    // fn write_byte(&mut self, val: u8);
    // fn write_i16(&mut self, val: i16);
    // fn write_i32(&mut self, val: i32);
    // fn write_i64(&mut self, val: i64);
    // fn write_str(&mut self, val: &str);
    // fn write_binary(&mut self, val: &[u8]);
}

pub struct BinaryProtocol<'a> {
    wr: Serializer<'a>
}

impl<'a> BinaryProtocol<'a> {
    pub fn new(wr: &'a mut Write) -> BinaryProtocol<'a> {
        BinaryProtocol {
            wr: Serializer::new(wr)
        }
    }
}

impl<'a> Protocol for BinaryProtocol<'a> {
    fn write_message_begin(&mut self, name: &str, message_type: ThriftMessageType) {
        let version = THRIFT_VERSION_1 | message_type as i32;

        self.wr.serialize_i32(version);
        self.wr.serialize_str(name);
        // Seqid is always 0 apparently.
        self.wr.serialize_i16(0);
    }
}

pub struct Serializer<'a> {
    wr: &'a mut Write
}

impl<'a> Serializer<'a> {
    pub fn new(wr: &'a mut Write) -> Serializer<'a> {
        Serializer {
            wr: wr
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Noop
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Foo"
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Noop
    }
}

impl<'a> serde::Serializer for Serializer<'a> {
    type Error = Error;

    fn serialize_unit(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_bool(&mut self, val: bool) -> Result<(), Error> {
        if val {
            self.wr.write_i8(1);
        } else {
            self.wr.write_i8(0);
        }

        Ok(())
    }

    fn serialize_u8(&mut self, val: u8) -> Result<(), Error> {
        self.serialize_i8(val as i8)
    }

    fn serialize_u16(&mut self, val: u16) -> Result<(), Error> {
        self.serialize_i16(val as i16)
    }

    fn serialize_u32(&mut self, val: u32) -> Result<(), Error> {
        self.serialize_i32(val as i32)
    }

    fn serialize_u64(&mut self, val: u64) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_usize(&mut self, val: usize) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_i8(&mut self, val: i8) -> Result<(), Error> {
        self.wr.write_i8(val);
        Ok(())
    }

    fn serialize_i16(&mut self, val: i16) -> Result<(), Error> {
        self.wr.write_i16::<BigEndian>(val);
        Ok(())
    }

    fn serialize_i32(&mut self, val: i32) -> Result<(), Error> {
        self.wr.write_i32::<BigEndian>(val);
        Ok(())
    }

    fn serialize_i64(&mut self, val: i64) -> Result<(), Error> {
        self.wr.write_i64::<BigEndian>(val);
        Ok(())
    }

    fn serialize_isize(&mut self, val: isize) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_f32(&mut self, val: f32) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_f64(&mut self, val: f64) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_char(&mut self, val: char) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_str(&mut self, val: &str) -> Result<(), Error> {
        self.wr.write(val.as_bytes());
        Ok(())
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_some<T>(&mut self, v: T) -> Result<(), Error>
        where T: Serialize
    {
        Ok(())
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        self.wr.write(value);
        Ok(())
    }

    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor
    {
        Ok(())
    }

    fn serialize_seq_elt<V>(&mut self, value: V) -> Result<(), Error>
        where V: serde::Serialize
    {
        value.serialize(self)
    }


    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor
    {
        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: serde::Serialize,
              V: serde::Serialize,
    {
        try!(key.serialize(self));
        value.serialize(self)
    }

}

#[cfg(test)]
mod tests {
    use serde::ser::Serializer;
    use std::io::{Cursor, Read};
    use byteorder::{ReadBytesExt, BigEndian};
    use super::{ThriftMessageType, BinaryProtocol, Protocol};

    #[test]
    fn serialize_bool_true() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_bool(true);
        }

        assert_eq!(v[0], 1);
    }

    #[test]
    fn serialize_bool_false() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_bool(false);
        }

        assert_eq!(v[0], 0);
    }

    #[test]
    fn serialize_i8() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i8(5);
        }

        assert_eq!(v[0], 5);
    }

    #[test]
    fn serialize_i8_neg() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i8(-5);
        }

        assert_eq!(v[0] as i8, -5);
    }

    #[test]
    fn serialize_i16() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i16(900);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(900, cursor.read_i16::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i16_neg() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i16(-900);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-900, cursor.read_i16::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i32() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i32(3000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(3000000, cursor.read_i32::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i32_neg() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i32(-3000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-3000000, cursor.read_i32::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i64() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i64(33000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(33000000, cursor.read_i64::<BigEndian>().unwrap());
    }

    #[test]
    fn serialize_i64_neg() {
        let mut v = Vec::new();
        {
            let mut s = super::Serializer::new(&mut v);
            s.serialize_i64(-33000000);
        }

        let mut cursor = Cursor::new(v);
        assert_eq!(-33000000, cursor.read_i64::<BigEndian>().unwrap());
    }

    #[test]
    fn protocol_begin() {
        let mut v = Vec::new();
        {
            let mut proto = BinaryProtocol::new(&mut v);
            proto.write_message_begin("foobar", ThriftMessageType::Call);
        }

        let mut cursor = Cursor::new(v);
        let version = super::THRIFT_VERSION_1 | super::ThriftMessageType::Call as i32;

        assert_eq!(version, cursor.read_i32::<BigEndian>().unwrap());
        // XXX Decode string and seqid.
    }
}
