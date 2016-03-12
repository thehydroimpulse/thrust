use std::io::{Read, Write};

pub trait Serializer {
    type Error = ();
    fn serialize_bool(&mut self, val: bool) -> Result<(), Self::Error>;
    fn serialize_str(&mut self, val: &str) -> Result<(), Self::Error>;
    fn serialize_string(&mut self, val: String) -> Result<(), Self::Error>;
    fn serialize_usize(&mut self, val: usize) -> Result<(), Self::Error>;
    fn serialize_isize(&mut self, val: isize) -> Result<(), Self::Error>;
    fn serialize_u64(&mut self, val: u64) -> Result<(), Self::Error>;
    fn serialize_i64(&mut self, val: i64) -> Result<(), Self::Error>;
    fn serialize_i32(&mut self, val: i32) -> Result<(), Self::Error>;
    fn serialize_u32(&mut self, val: u32) -> Result<(), Self::Error>;
    fn serialize_i16(&mut self, val: i16) -> Result<(), Self::Error>;
    fn serialize_u16(&mut self, val: u16) -> Result<(), Self::Error>;
    fn serialize_u8(&mut self, val: u8) -> Result<(), Self::Error>;
    fn serialize_i8(&mut self, val: i8) -> Result<(), Self::Error>;
    fn serialize_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error>;
}

pub trait Serialize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error> where S: Serializer + ThriftSerializer;
}

pub trait ThriftSerializer {
    type TError = ();

    fn write_message_begin(&mut self, name: &str) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_struct_begin(&mut self, name: &str) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_struct_end(&mut self) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_field_begin(&mut self, name: &str, ty: u16, id: u16) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_field_end(&mut self) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_field_stop(&mut self) -> Result<(), Self::TError> {
        Ok(())
    }

    fn write_message_end(&mut self) -> Result<(), Self::TError> {
        Ok(())
    }
}

impl Serialize for bool {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_bool(*self)
    }
}

impl<'a> Serialize for &'a str {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_str(self)
    }
}

impl Serialize for String {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_string(self.clone())
    }
}

impl Serialize for usize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_usize(*self)
    }
}

impl Serialize for isize {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_isize(*self)
    }
}

impl Serialize for u64 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u64(*self)
    }
}

impl Serialize for i64 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i64(*self)
    }
}

impl Serialize for i32 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i32(*self)
    }
}

impl Serialize for u32 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u32(*self)
    }
}

impl Serialize for u16 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u16(*self)
    }
}

impl Serialize for i16 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i16(*self)
    }
}

impl Serialize for i8 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_i8(*self)
    }
}

impl Serialize for u8 {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_u8(*self)
    }
}

impl<'a> Serialize for &'a [u8] {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer + ThriftSerializer
    {
        s.serialize_bytes(self)
    }
}
