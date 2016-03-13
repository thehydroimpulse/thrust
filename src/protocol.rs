use std::io::{self, Read, Write};
use std::convert;
use byteorder;
use std::string::FromUtf8Error;

pub enum Error {
    Byteorder(byteorder::Error),
    Io(io::Error),
    Utf8Error(FromUtf8Error),
    BadVersion,
    ProtocolVersionMissing
}

impl convert::From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        Error::Byteorder(err)
    }
}

impl convert::From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ThriftType {
    Stop = 0,
    Void = 1,
    Bool = 2,
    Byte = 3,
    Double = 4,
    I16 = 6,
    I32 = 8,
    U64 = 9,
    I64 = 10,
    String = 11,
    Struct = 12,
    Map = 13,
    Set = 14,
    List = 15
}

impl convert::From<i8> for ThriftType {
    fn from(val: i8) -> ThriftType {
        match val {
            0 => ThriftType::Stop,
            1 => ThriftType::Void,
            2 => ThriftType::Bool,
            3 => ThriftType::Byte,
            4 => ThriftType::Double,
            6 => ThriftType::I16,
            8 => ThriftType::I32,
            9 => ThriftType::U64,
            10 => ThriftType::I64,
            11 => ThriftType::String,
            12 => ThriftType::Struct,
            13 => ThriftType::Map,
            14 => ThriftType::Set,
            15 => ThriftType::List,
            _ => panic!("Unexpected value")
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ThriftMessageType {
    Call = 1,
    Reply = 2,
    Exception = 3,
    Oneway = 4
}

impl convert::From<i8> for ThriftMessageType {
    fn from(val: i8) -> ThriftMessageType {
        match val {
            1 => ThriftMessageType::Call,
            2 => ThriftMessageType::Reply,
            3 => ThriftMessageType::Exception,
            4 => ThriftMessageType::Oneway,
            _ => panic!("Unexpected value for ThriftMessageType.")
        }
    }
}

pub trait Serializer {
    type Error = ();

    fn serialize_bool(&mut self, val: bool) -> Result<(), Error>;
    fn serialize_str(&mut self, val: &str) -> Result<(), Error>;
    fn serialize_string(&mut self, val: String) -> Result<(), Error>;
    fn serialize_usize(&mut self, val: usize) -> Result<(), Error>;
    fn serialize_isize(&mut self, val: isize) -> Result<(), Error>;
    fn serialize_u64(&mut self, val: u64) -> Result<(), Error>;
    fn serialize_i64(&mut self, val: i64) -> Result<(), Error>;
    fn serialize_i32(&mut self, val: i32) -> Result<(), Error>;
    fn serialize_u32(&mut self, val: u32) -> Result<(), Error>;
    fn serialize_i16(&mut self, val: i16) -> Result<(), Error>;
    fn serialize_u16(&mut self, val: u16) -> Result<(), Error>;
    fn serialize_u8(&mut self, val: u8) -> Result<(), Error>;
    fn serialize_i8(&mut self, val: i8) -> Result<(), Error>;
    fn serialize_bytes(&mut self, val: &[u8]) -> Result<(), Error>;
}

pub trait Deserializer {
    type Error = ();

    fn deserialize_bool(&mut self) -> Result<bool, Error>;
    fn deserialize_usize(&mut self) -> Result<usize, Error>;
    fn deserialize_isize(&mut self) -> Result<isize, Error>;
    fn deserialize_u64(&mut self) -> Result<u64, Error>;
    fn deserialize_i64(&mut self) -> Result<i64, Error>;
    fn deserialize_u32(&mut self) -> Result<u32, Error>;
    fn deserialize_i32(&mut self) -> Result<i32, Error>;
    fn deserialize_u16(&mut self) -> Result<u16, Error>;
    fn deserialize_i16(&mut self) -> Result<i16, Error>;
    fn deserialize_u8(&mut self) -> Result<u8, Error>;
    fn deserialize_i8(&mut self) -> Result<i8, Error>;
    fn deserialize_bytes(&mut self) -> Result<Vec<u8>, Error>;
    fn deserialize_str(&mut self) -> Result<String, Error>;
}

pub trait Serialize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error> where S: Serializer + ThriftSerializer;
}

pub trait ThriftSerializer {
    type TError = ();

    fn write_message_begin(&mut self, name: &str, message_type: ThriftMessageType) -> Result<(), Error> {
        Ok(())
    }

    fn write_struct_begin(&mut self, name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn write_struct_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_field_begin(&mut self, name: &str, ty: ThriftType, id: i16) -> Result<(), Error> {
        Ok(())
    }

    fn write_field_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_field_stop(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write_message_end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct ThriftMessage {
    pub name: String,
    pub ty: ThriftMessageType,
    pub seq: i16
}

pub struct ThriftField {
    pub name: Option<String>,
    pub ty: ThriftType,
    pub seq: i16
}

pub trait ThriftDeserializer {
    type TError = ();

    fn read_message_begin(&mut self) -> Result<ThriftMessage, Error>;
    fn read_message_end(&mut self) -> Result<(), Error>;
    fn read_struct_begin(&mut self) -> Result<String, Error>;
    fn read_struct_end(&mut self) -> Result<(), Error>;
    fn read_field_begin(&mut self) -> Result<ThriftField, Error>;
    fn read_field_end(&mut self) -> Result<(), Error>;
}

impl Serialize for bool {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_bool(*self)
    }
}

impl<'a> Serialize for &'a str {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_str(self)
    }
}

impl Serialize for String {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_string(self.clone())
    }
}

impl Serialize for usize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_usize(*self)
    }
}

impl Serialize for isize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_isize(*self)
    }
}

impl Serialize for u64 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u64(*self)
    }
}

impl Serialize for i64 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i64(*self)
    }
}

impl Serialize for i32 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i32(*self)
    }
}

impl Serialize for u32 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u32(*self)
    }
}

impl Serialize for u16 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u16(*self)
    }
}

impl Serialize for i16 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i16(*self)
    }
}

impl Serialize for i8 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i8(*self)
    }
}

impl Serialize for u8 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u8(*self)
    }
}

impl<'a> Serialize for &'a [u8] {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_bytes(self)
    }
}
