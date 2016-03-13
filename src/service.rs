use protocol::{Serializer, ThriftSerializer, ThriftMessage, ThriftDeserializer, Deserializer,
Deserialize, Serialize, ThriftType, ThriftMessageType, Error};
use binary_protocol::{BinarySerializer, BinaryDeserializer};
use std::io::Cursor;

pub trait Service {
    fn query(&mut self, val: bool);
}

/// TransformCall trait is the transformation from a thrift object to
/// a Rust method call.
pub struct TransformIncomingCall<'a, D: 'a> {
    de: &'a mut D,
    service: &'a mut Service
}

pub trait TransformCall {
    fn call_query(&mut self) -> Result<(), Error>;
}

/// XXX: Add support for RPC return/reply.
/// XXX: Add Future support for RPC calls. This will allow us to support return types as we can
/// essentially have:
///
/// ```notrust
/// TransformIncomingCall::new(&mut de).call_query()
/// ```
impl<'a, D> TransformIncomingCall<'a, D>
    where D: 'a + Deserializer + ThriftDeserializer
{
    pub fn new(de: &'a mut D, service: &'a mut Service) -> TransformIncomingCall<'a, D> {
        TransformIncomingCall {
            de: de,
            service: service
        }
    }
}

impl<'a, D> TransformCall for TransformIncomingCall<'a, D>
    where D: 'a + Deserializer + ThriftDeserializer
{
    fn call_query(&mut self) -> Result<(), Error> {
        // Deserialize into QueryArgs
        let args: QueryArgs = try!(Deserialize::deserialize(self.de));

        self.service.query(args.val);
        Ok(())
    }
}

// Generated
fn transform_msg<D>(msg: ThriftMessage, de: &mut D, service: &mut Service) -> Result<(), Error>
    where D: Deserializer + ThriftDeserializer
{
    match &*msg.name {
        "query" => {
            let args: QueryArgs = try!(Deserialize::deserialize(de));
            service.query(args.val);
        },
        _ => {
            // Return Err.
        }
    }

    Ok(())
}

pub struct QueryArgs {
    val: bool
}

impl Deserialize for QueryArgs {
    fn deserialize<D>(de: &mut D) -> Result<Self, Error>
        where D: Deserializer + ThriftDeserializer
    {
        Ok(QueryArgs {
            val: try!(de.deserialize_bool())
        })
    }
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

pub struct RpcClient {
    buf: Vec<u8>
}

impl RpcClient {
    pub fn new() -> RpcClient {
        RpcClient {
            buf: Vec::new()
        }
    }
}

impl Service for RpcClient {
    fn query(&mut self, val: bool) {
        {
            let mut proto = BinarySerializer::new(&mut self.buf);
            let args = QueryArgs {
                val: val
            };

            proto.write_message_begin("query", ThriftMessageType::Call);
            args.serialize(&mut proto);
            proto.write_message_end();
        }
    }
}

pub struct MessagePipeline<'a, D> {
    de: D,
    service: &'a mut Service
}

impl<'a, D> MessagePipeline<'a, D>
    where D: Deserializer + ThriftDeserializer
{
    pub fn new(de: D, service: &'a mut Service) -> MessagePipeline<D> {
        MessagePipeline {
            de: de,
            service: service
        }
    }

    /// Dispatch the incoming RPC call to the respective service method.
    pub fn dispatch(&mut self, msg: ThriftMessage) -> Result<(), Error> {
        try!(transform_msg(msg, &mut self.de, self.service));
        Ok(())
    }

    /// XXX: The fn signature should be `Result<Future<Vec<u8>>, Error>` where the serialized
    /// response is returned into the future.
    pub fn run(&mut self) -> Result<(), Error> {
        let msg = try!(self.de.read_message_begin());

        match msg.ty {
            // Dispatch on an RPC method call.
            ThriftMessageType::Call => {
                try!(self.dispatch(msg));
            },
            _ => {}
        }

        Ok(())
    }
}

#[test]
fn call_query() {
    let mut buf = {
        let mut rpc = RpcClient::new();
        rpc.query(true);
        rpc.buf
    };

    struct Server;

    impl Service for Server {
        fn query(&mut self, val: bool) {
            assert_eq!(val, true);
        }
    }

    let mut de = BinaryDeserializer::new(Cursor::new(buf));
    let mut s = Server;
    let mut pipe = MessagePipeline::new(de, &mut s);
    // XXX: Expect a future as return value.
    //
    // ```notrust
    // pipe.run().and_then(|res| {
    //     // ...
    // })
    // ```
    //
    // Where `res` is the serialized response.
    pipe.run().unwrap();
}
