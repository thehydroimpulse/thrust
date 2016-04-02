use protocol::*;
use binary_protocol::*;

pub fn create_empty_thrift_message(method: &str, ty: ThriftMessageType) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut se = BinarySerializer::new(&mut buf);
        se.write_message_begin(method, ty);
        se.write_message_end();
    }

    buf
}

