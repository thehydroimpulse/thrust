use protocol::{Serializer, ThriftSerializer};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use byteorder;

pub enum Error {
    Noop
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
        Ok(())
    }

    fn serialize_u64(&mut self, val: u64) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_i64(&mut self, val: i64) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_u32(&mut self, val: u32) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_i32(&mut self, val: i32) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_u16(&mut self, val: u16) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_i16(&mut self, val: i16) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_u8(&mut self, val: u8) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_i8(&mut self, val: i8) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_bytes(&mut self, val: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_str(&mut self, val: &str) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_string(&mut self, val: String) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_usize(&mut self, val: usize) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_isize(&mut self, val: isize) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ThriftSerializer for BinarySerializer<'a> {

}
