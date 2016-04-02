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
use std::thread::JoinHandle;
use reactor::Reactor;

pub trait Service : Send {
    fn query(&mut self, val: bool) -> Future<()>;
}

pub struct ServiceRunner<S: Service> {
    service: S
}

impl<S: Service> ServiceRunner<S> {
    pub fn new(service: S) -> ServiceRunner<S> {
        ServiceRunner {
            service: service
        }
    }
}

impl<D, S: Service> Runner<D> for ServiceRunner<S>
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
    dispatcher: Sender<Message>,
    handle: JoinHandle<()>
}

impl RpcClient {
    pub fn new(addr: SocketAddr) -> RpcClient {
        let (handle, tx) = dispatch(addr);

        RpcClient {
            dispatcher: tx,
            handle: handle
        }
    }

    fn join(mut self) {
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
        let mut buf = Vec::new();

        {
            let mut proto = BinarySerializer::new(&mut buf);
            let args = QueryArgs {
                val: val
            };

            proto.write_message_begin("query", ThriftMessageType::Call);
            args.serialize(&mut proto);
            proto.write_message_end();
        }

        self.dispatcher.send(Message::Call("query".to_string(), buf, tx));

        future.map(|v| {

            ()
        })
    }
}

#[test]
fn call_query() {
    struct S;

    impl Service for S {
        fn query(&mut self, val: bool) -> Future<()> {
            assert_eq!(val, true);
            Future::unit(())
        }
    }

    let run = ServiceRunner::new(S);
    let mut server = Server::new(run);
    server.bind("0.0.0.0:9455".parse().unwrap());

    let addr = "0.0.0.0:9455".parse().unwrap();
    let mut rpc = RpcClient::new(addr);

    rpc.query(true);

    // {
    //     let mut de = BinaryDeserializer::new(Cursor::new(buf));
    //     let mut s = Server;
    //     let mut caller = ServiceRunner::new(&mut s);
    //     MessagePipeline::new(de).run(&mut caller).unwrap().and_then(|v| {
    //         println!("#query result: {:?}", v);
    //         Async::Ok(())
    //     });
    // }

    Reactor::run();
    rpc.shutdown();
}
