use protocol::{Serializer, ThriftSerializer, Serialize, ThriftType, ThriftMessageType, Error};
use binary_protocol::{BinarySerializer, BinaryDeserializer};

pub trait Service {
    fn query(&mut self, val: bool);
}

pub struct QueryArgs {
    val: bool
}

impl Serialize for QueryArgs {
    fn serialize<S>(&self, s: &mut S) -> Result<(), Error>
        where S: Serializer + ThriftSerializer
    {
        try!(s.write_struct_begin("query_args"));

        // for each field
        try!(s.write_field_begin("val", ThriftType::Bool, 1));
        try!(self.val.serialize(s));
        try!(s.write_field_stop());
        try!(s.write_field_end());

        try!(s.write_struct_end());

        Ok(())
    }
}


pub struct RpcServer;

impl Service for RpcServer {
    fn query(&mut self, val: bool) {

    }
}

fn dispatch_query(service: &mut Service, args: QueryArgs) {
    service.query(args.val);
}

pub struct RpcClient;

impl Service for RpcClient {
    fn query(&mut self, val: bool) {
        let mut v = Vec::new();

        {
            let mut proto = BinarySerializer::new(&mut v);
            let args = QueryArgs {
                val: val
            };

            proto.write_message_begin("query", ThriftMessageType::Call);
            args.serialize(&mut proto);
            proto.write_message_end();
        }
    }
}
