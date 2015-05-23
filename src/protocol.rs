use std::io;
use serde::ser::{Serializer, Serialize, SeqVisitor, MapVisitor};

pub trait Protocol {}

pub struct BinarySerializer<W> {
    writer: W
}

impl<W> BinarySerializer<W>
    where W: io::Write
{
    #[inline]
    pub fn new(writer: W) -> Self {
        BinarySerializer {
            writer: writer
        }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[inline]
pub fn to_bytes<T>(value: &T) -> Vec<u8>
    where T: Serialize
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value).unwrap();
    writer
}


#[inline]
pub fn to_writer<T, W>(writer: &mut W, value: &T) -> io::Result<()>
    where T: Serialize,
          W: io::Write
{
    let mut ser = BinarySerializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

impl<W> Serializer for BinarySerializer<W>
    where W: io::Write
{
    type Error = io::Error;

    #[inline]
    fn visit_bool(&mut self, value: bool) -> io::Result<()> {
        if value {
            self.writer.write_all(&[1])
        } else {
            self.writer.write_all(&[0])
        }
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> io::Result<()> {
        let mut out: [u8; 8] = [0; 8];
        out[0] = (0xff & (value >> 56)) as u8;
        out[1] = (0xff & (value >> 48)) as u8;
        out[2] = (0xff & (value >> 40)) as u8;
        out[3] = (0xff & (value >> 32)) as u8;
        out[4] = (0xff & (value >> 24)) as u8;
        out[5] = (0xff & (value >> 16)) as u8;
        out[6] = (0xff & (value >> 8)) as u8;
        out[7] = (0xff & (value)) as u8;

        self.writer.write_all(&out[..])
    }

    #[inline]
    fn visit_bytes(&mut self, value: &[u8]) -> io::Result<()> {
        let len = value.len();
        try!(self.visit_i32(len as i32));
        self.writer.write_all(value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> io::Result<()> {
        let mut out: [u8; 4] = [0; 4];
        out[0] = (0xff & (value >> 24)) as u8;
        out[1] = (0xff & (value >> 16)) as u8;
        out[2] = (0xff & (value >> 8)) as u8;
        out[3] = (0xff & (value)) as u8;

        self.writer.write_all(&out[..])
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> io::Result<()> {
        self.visit_i64(value as i64)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> io::Result<()> {
        self.visit_i64(value as i64)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();
        self.visit_i32(bytes.len() as i32);
        self.writer.write_all(bytes)
    }

    #[inline]
    fn visit_unit(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn visit_some<V>(&mut self, value: V) -> io::Result<()>
        where V: Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn visit_none(&mut self) -> io::Result<()> {
        self.visit_unit()
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> io::Result<()> {
        let mut out: [u8; 2] = [0; 2];

        out[0] = (0xff & (value >> 8)) as u8;
        out[1] = (0xff & (value)) as u8;

        self.writer.write_all(&out[..])
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: SeqVisitor
    {
        match visitor.len() {
            Some(len) if len == 0 => {

            },
            _ => {

            }
        }

        Ok(())
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self, _name: &str, variant: &str, visitor: V) -> io::Result<()>
        where V: SeqVisitor
    {
        Ok(())
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, value: T) -> io::Result<()>
        where T: Serialize,
    {
        Ok(())
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> io::Result<()>
        where V: MapVisitor,
    {
        Ok(())
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> io::Result<()>
        where K: Serialize,
              V: Serialize,
    {
        Ok(())
    }

    #[inline]
    fn format() -> &'static str {
        "binary"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Serialize;
    use std::io::{Cursor, Read};
    use byteorder::{ReadBytesExt, BigEndian};
    use std::str;

    #[derive(Serialize)]
    struct Metadata {
        source_id: i64,
        state_id: i32,
        count: i32,
        updated_at: i32
    }

    #[test]
    fn serialize_bool_true_manual() {
        let mut writer = Vec::with_capacity(5);
        let mut ser = BinarySerializer::new(writer);
        true.serialize(&mut ser).unwrap();
        let vec = ser.into_inner();
        assert_eq!(vec.len(), 1);
        assert_eq!(&*vec, &[1]);
    }

    #[test]
    fn serialize_bool_false() {
        assert_eq!(&*to_bytes(&false), &[0]);
    }

    #[test]
    fn serialize_str() {
        let mut bytes = Cursor::new(to_bytes(&"foobar"));

        // The length is encoded within an i32 integer.
        let mut len = [0u8; 4];
        bytes.read(&mut len).unwrap();

        // Decode the length slice back into an i32 integer and validate that
        // the length is correct.
        let len = (&len[..]).read_i32::<BigEndian>().unwrap();
        assert_eq!(6, len);

        // Read in the encoded UTF-8 string.
        let mut buf = [0u8; 6];
        bytes.read(&mut buf).unwrap();

        assert_eq!("foobar", str::from_utf8((&buf[..])).unwrap());
    }
}
