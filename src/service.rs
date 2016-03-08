use protocol::{Protocol, BinaryProtocol, ThriftType, ThriftMessageType};

pub trait Service {
    fn query(&mut self, val: &str);
}

pub struct RpcClient;

impl Service for RpcClient {
    fn query(&mut self, val: &str) {
        let mut v = Vec::new();

        {
            let mut proto = BinaryProtocol::new(&mut v);

            proto.write_message_begin("query", ThriftMessageType::Call);
            proto.write_struct_begin("query_args");

            proto.write_field_begin("q", ThriftType::String, 1);
            proto.write_str(val);
            proto.write_field_stop();

            proto.write_field_end();
            proto.write_struct_end();

            // XXX: Write the arguments with `write_field_begin`
            proto.write_message_end();
        }
    }
}
