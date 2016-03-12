use protocol::{Serializer, ThriftSerializer};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use byteorder;
use std::convert;

pub enum Error {
    Byteorder(byteorder::Error),
    Io(io::Error)
}

impl convert::From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        Error::Byteorder(err)
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

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
    type Error = Error;

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

}
