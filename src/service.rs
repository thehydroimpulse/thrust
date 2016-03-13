use protocol::{Serializer, ThriftSerializer, ThriftMessage, ThriftDeserializer, Deserializer,
Deserialize, Serialize, ThriftType, ThriftMessageType, Error};
use binary_protocol::{BinarySerializer, BinaryDeserializer};
use std::io::Cursor;
use tangle::{Future, Async};
use pipeline::MessagePipeline;
use message_dispatcher::Dispatcher;

pub trait Service {
    fn query(&mut self, val: bool) -> Future<()>;
}

pub struct DispatchService<'a> {
    service: &'a mut Service
}

impl<'a> DispatchService<'a> {
    pub fn new(service: &'a mut Service) -> DispatchService<'a> {
        DispatchService {
            service: service
        }
    }
}

impl<'a, D> Dispatcher<D> for DispatchService<'a>
    where D: Deserializer + ThriftDeserializer
{
    fn call(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error> {
        match &*msg.name {
            "query" => {
                let args: QueryArgs = try!(Deserialize::deserialize(de));
                Ok(self.service.query(args.val).map(|val| {
                    let mut v = Vec::new();
                    {
                        let mut s = BinarySerializer::new(&mut v);
                        s.write_message_begin("query", ThriftMessageType::Reply);

                        s.write_struct_begin("query_ret");

                        s.write_field_begin("ret", ThriftType::Void, 1);
                        s.write_field_stop();
                        s.write_field_end();

                        s.write_struct_end();

                        s.write_message_end();
                    }

                    v
                }))
            },
            _ => {
                unimplemented!()
                // Return Err.
            }
        }
    }
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
    fn query(&mut self, val: bool) -> Future<()> {
        {
            let mut proto = BinarySerializer::new(&mut self.buf);
            let args = QueryArgs {
                val: val
            };

            proto.write_message_begin("query", ThriftMessageType::Call);
            args.serialize(&mut proto);
            proto.write_message_end();
        }

        Future::unit(())
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
        fn query(&mut self, val: bool) -> Future<()> {
            assert_eq!(val, true);
            Future::unit(())
        }
    }

    let mut de = BinaryDeserializer::new(Cursor::new(buf));
    let mut s = Server;
    let mut pipe = MessagePipeline::new(de);
    // XXX: Expect a future as return value.
    //
    // ```notrust
    // pipe.run().and_then(|res| {
    //     // ...
    // })
    // ```
    //
    // Where `res` is the serialized response.
    let mut dispatcher = DispatchService::new(&mut s);
    pipe.run(&mut dispatcher).unwrap().and_then(|v| {
        println!("{:?}", v);
        Async::Ok(())
    });
}
