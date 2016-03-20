use std::io::Cursor;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::net::SocketAddr;
use protocol::{Serializer, ThriftSerializer, ThriftMessage, ThriftDeserializer, Deserializer,
Deserialize, Serialize, ThriftType, ThriftMessageType, Error};
use binary_protocol::{BinarySerializer, BinaryDeserializer};
use tangle::{Future, Async};
use server::{Server};

use pipeline::MessagePipeline;
use runner::Runner;
use dispatcher::{Message, dispatch};
use request::Request;
use std::thread::JoinHandle;

pub trait Service {
    fn query(&mut self, val: bool) -> Future<()>;
}

pub struct ServiceRunner<'a> {
    service: &'a mut Service
}

impl<'a> ServiceRunner<'a> {
    pub fn new(service: &'a mut Service) -> ServiceRunner<'a> {
        ServiceRunner {
            service: service
        }
    }
}

impl<'a, D> Runner<D> for ServiceRunner<'a>
    where D: Deserializer + ThriftDeserializer
{
    fn run(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error> {
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

pub struct RpcClient {
    buf: Vec<u8>,
    dispatcher: Sender<Message>,
    handle: JoinHandle<()>
}

impl RpcClient {
    pub fn new(addr: SocketAddr) -> RpcClient {
        let (handle, tx) = dispatch(addr);

        RpcClient {
            buf: Vec::new(),
            dispatcher: tx,
            handle: handle
        }
    }

    pub fn join(mut self) {
        self.handle.join();
    }

    pub fn shutdown(mut self) {
        self.dispatcher.send(Message::Shutdown);
        self.join();
    }
}

impl Service for RpcClient {
    fn query(&mut self, val: bool) -> Future<()> {
        let (tx, rx) = channel();
        let future = Future::<Vec<u8>>::from_channel(rx);

        {
            let mut proto = BinarySerializer::new(&mut self.buf);
            let args = QueryArgs {
                val: val
            };

            proto.write_message_begin("query", ThriftMessageType::Call);
            args.serialize(&mut proto);
            proto.write_message_end();
        }

        let req = Request::new(self.buf.clone());
        self.dispatcher.send(Message::Req(req, tx));

        future.map(|v| ())
    }
}

#[test]
fn call_query() {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let mut rpc = RpcClient::new(addr);
    rpc.query(true);
    let buf = rpc.buf.clone();

    struct Server;

    impl Service for Server {
        fn query(&mut self, val: bool) -> Future<()> {
            assert_eq!(val, true);
            Future::unit(())
        }
    }

    {
        let mut de = BinaryDeserializer::new(Cursor::new(buf));
        let mut s = Server;
        let mut caller = ServiceRunner::new(&mut s);
        MessagePipeline::new(de).run(&mut caller).unwrap().and_then(|v| {
            println!("#query result: {:?}", v);
            Async::Ok(())
        });
    }

    rpc.shutdown();
}
