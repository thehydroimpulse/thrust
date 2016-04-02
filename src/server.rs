use runner::Runner;
use protocol::{Deserializer, ThriftDeserializer, Error};
use binary_protocol::BinaryDeserializer;
use std::io::Cursor;
use event_loop::SENDERS;
use reactor::{Reactor, Message, Dispatch};
use std::thread;
use mio;
use tangle::{Future, Async};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::net::{SocketAddr, TcpListener};

pub type Default = BinaryDeserializer<Cursor<Vec<u8>>>;

/// Manages incoming RPC requests from a Mio event loop and dispatches it
/// to a runner that will then deserialize the Thrift message and call the appropriate
/// RPC function.
///
/// The server will also manage the response coming back from the RPC method through
/// the use of futures. These will be coordinated back to Mio.
pub struct Server<R: Runner<Default> + Send> {
    senders: Vec<mio::Sender<Message>>,
    de: Default,
    runner: R
}

fn create_task<R: Runner<Default> + Send>(mut server: Server<R>, rx: Receiver<Dispatch>) -> Result<(), Error> {
    for msg in rx.iter() {

        match msg {
            // We have an incoming RPC request that we need to decode. We need to decode
            // this information and later send a response with the same token.
            Dispatch::Data(reactor_id, token, buf) => {
                let mut de = BinaryDeserializer::new(Cursor::new(buf));
                let thrift_msg = try!(de.read_message_begin());

                // We want to send the reply RPC call to the same event loop where
                // the original call came from.
                let sender = server.senders[reactor_id].clone();
                try!(server.runner.run(&mut de, thrift_msg)).and_then(move |buf| {
                    // Pass the reply back to the original sender.
                    sender.send(Message::Rpc(token, buf));
                    Async::Ok(())
                });
            },
            Dispatch::Id(token) => {}
        }
    }

    Ok(())
}

impl<R> Server<R>
    where R: 'static + Runner<Default> + Send
{
    pub fn new(runner: R) -> Server<R> {
        Server {
            senders: SENDERS.clone(),
            de: BinaryDeserializer::new(Cursor::new(Vec::new())),
            runner: runner
        }
    }

    pub fn bind(mut self, addr: SocketAddr) {
        thread::spawn(move || {
            // Local channel for the mio event loops to communicate with.
            let (tx, rx) = channel();

            // Send the listener along with a receiving channel to each of the
            // Mio event loops.
            for (id, sender) in self.senders.iter().enumerate() {
                sender.send(Message::Bind(id, addr, tx.clone()));
            }

            match create_task(self, rx) {
                Ok(()) => {},
                Err(err) => panic!("{:?}", err)
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reactor::{Reactor, Message, Dispatch};
    use mio::{Token, EventLoop};
    use runner::Runner;
    use std::time::Duration;
    use std::io::Cursor;
    use binary_protocol::{BinarySerializer, BinaryDeserializer};
    use protocol::{Deserializer, ThriftMessage, ThriftSerializer, ThriftDeserializer, ThriftMessageType, Error};
    use std::thread;
    use tangle::{Future, Async};
    use std::sync::mpsc::channel;
    use event_loop::SENDERS;

    struct TestRunner;

    impl<D> Runner<D> for TestRunner
        where D: Deserializer + ThriftDeserializer
    {
        fn run(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error> {
            let mut buf = Vec::new();

            {
                let mut se = BinarySerializer::new(&mut buf);
                se.write_message_begin("foobar123", ThriftMessageType::Reply);
                se.write_message_end();
            }

            Ok(Future::unit(buf))
        }
    }

    #[test]
    fn init() {
        let server = Server::new(TestRunner);

        server.bind("127.0.0.1:3456".parse().unwrap());

        let clients = SENDERS.clone();

        let (tx, rx) = channel();
        thread::sleep(Duration::from_millis(10));
        clients[0].send(Message::Connect(0, "127.0.0.1:3456".parse().unwrap(), tx));

        thread::spawn(move || {
            let mut i = 0;
            for msg in rx.iter() {
                match msg {
                    Dispatch::Id(token) => {
                        if i > 0 { continue }

                        i += 1;

                        let mut buf = Vec::new();

                        {
                            let mut se = BinarySerializer::new(&mut buf);
                            se.write_message_begin("foobar123", ThriftMessageType::Call);
                            se.write_message_end();
                        }

                        clients[0].send(Message::Rpc(token, buf));
                    },
                    Dispatch::Data(reactor_id, token, buf) => {
                        let mut de = BinaryDeserializer::new(Cursor::new(buf));
                        let thrift_msg = de.read_message_begin().unwrap();

                        assert_eq!(thrift_msg.name, "foobar123");
                        assert_eq!(thrift_msg.ty, ThriftMessageType::Reply);

                        for client in clients.iter() {
                            client.send(Message::Shutdown);
                        }
                    }
                }
            }
        });

        Reactor::run();
    }
}
