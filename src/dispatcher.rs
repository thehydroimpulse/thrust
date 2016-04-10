use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use event_loop::SENDER;
use std::net::SocketAddr;
use mio::{self, Token};
use result::{ThrustResult, ThrustError};
use tangle::{Future, Async};
use std::io::Cursor;
use protocol::*;
use binary_protocol::*;
use reactor::{self, Dispatch, Message, Id};
use util;
use runner::Runner;

pub enum Role {
    /// A server will be tasked with actually calling a user defined
    /// RPC method and dispatching the response back to the event loop.
    Server(SocketAddr, Sender<(Token, Vec<u8>)>),
    /// A client is tasked with sending an initial RPC and dispatching a response.
    ///
    Client(SocketAddr)
}

pub enum Incoming {
    /// Method name, data buf, and response channel.
    Call(String, Vec<u8>, Option<Sender<(ThriftMessage, BinaryDeserializer<Cursor<Vec<u8>>>)>>),
    Reply(Token, Vec<u8>),
    Shutdown
}

/// A middleman between incoming and outgoing messages from the event loop and
/// clients or servers. Each instance of a server or client has it's own Dispatcher.
///
/// Dispatchers run in their own thread and only expose a channel interface. This makes it
/// extremely easy to do multi-threading by simply cloning the dispatcher.
pub struct Dispatcher {
    role: Role,
    /// The connection token as used and exposed by the event loop. This is required
    /// to know where to send and receive Rpc calls.
    token: Token,
    data_rx: Receiver<Dispatch>,
    /// The channel to communicate with the event loop.
    event_loop: mio::Sender<Message>,
    rx: Receiver<Incoming>,
    /// The response queue that is used to match up outgoing requests with future
    /// responses. Each response has it's own sender channel.
    queue: HashMap<String, Sender<(ThriftMessage, BinaryDeserializer<Cursor<Vec<u8>>>)>>
}

impl Dispatcher {
    pub fn spawn(role: Role) -> ThrustResult<(JoinHandle<ThrustResult<()>>, Sender<Incoming>)> {
        let (ret_tx, ret_rx) = channel();
        let handle = thread::spawn(move || {
            let (sender, receiver) = channel();
            ret_tx.send(sender);

            let (id_tx, id_rx) = channel();
            let event_loop_sender = SENDER.clone();
            let (data_tx, data_rx) = channel();

            match &role {
                &Role::Server(addr, ref method_dispatch) => {
                    event_loop_sender.send(Message::Bind(addr, id_tx, data_tx))?;
                },
                &Role::Client(addr) => {
                    event_loop_sender.send(Message::Connect(addr, id_tx, data_tx))?;
                }
            }

            let Id(token) = id_rx.recv()?;

            Dispatcher {
                role: role,
                token: token,
                data_rx: data_rx,
                event_loop: event_loop_sender,
                rx: receiver,
                queue: HashMap::new()
            }.run()
        });

        Ok((handle, ret_rx.recv()?))
    }

    pub fn run(mut self) -> ThrustResult<()> {
        let rx = self.rx;
        let event_loop_rx = self.data_rx;

        loop {
            select! {
                user_msg = rx.recv() => {
                    match user_msg {
                        Ok(Incoming::Shutdown) => break,
                        Ok(Incoming::Call(method, buf, tx)) => {
                            self.event_loop.send(Message::Rpc(self.token, buf));
                            match tx {
                                Some(tx) => {
                                    self.queue.insert(method, tx);
                                },
                                None => {}
                            }
                        },
                        Ok(Incoming::Reply(token, buf)) => {
                            self.event_loop.send(Message::Rpc(token, buf));
                        },
                        // The sender-part of the channel has been disconnected.
                        Err(err) => break
                    }
                },
                event_loop_msg = event_loop_rx.recv() => {
                    match event_loop_msg {
                        Ok(Dispatch::Data(token, buf)) => {
                            println!("[dispatcher]: reading data from {:?}", token);
                            match self.role {
                                // Received an RPC call
                                Role::Server(_, ref sender) => {
                                    try!(sender.send((token, buf)));
                                },
                                // Received a reply RPC call
                                Role::Client(_) => {
                                    let mut de = BinaryDeserializer::new(Cursor::new(buf));
                                    let msg = de.read_message_begin()?;

                                    match self.queue.remove(&msg.name) {
                                        Some(tx) => {
                                            println!("[dispatcher/client]: reply received.");
                                            tx.send((msg, de))?;
                                        },
                                        None => { println!("Cannot find {:?} method name", msg.name); }
                                    }
                                }
                            }
                        },
                        Err(err) => break
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tangle::{Future, Async};
    use std::net::SocketAddr;
    use std::io::Cursor;
    use reactor::{Reactor, Message};
    use event_loop::SENDER;
    use protocol::{ThriftMessage, ThriftMessageType};
    use binary_protocol::BinaryDeserializer;
    use std::sync::mpsc::channel;
    use util;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn should_create_server_dispatcher() {
        let addr = "127.0.0.1:5495".parse().unwrap();
        let (tx, rx) = channel();
        let (handle, tx) = Dispatcher::spawn(Role::Server(addr, tx)).unwrap();
    }

    #[test]
    fn should_start_server() {
        let addr: SocketAddr = "127.0.0.1:5955".parse().unwrap();
        let (method_dispatch_tx, method_dispatch_rx) = channel();
        let (handle_server, server) = Dispatcher::spawn(Role::Server(addr.clone(), method_dispatch)).unwrap();
        thread::sleep(Duration::from_millis(30));
        let (handle_client, client) = Dispatcher::spawn(Role::Client(addr.clone())).unwrap();

        let buf = util::create_empty_thrift_message("foobar123", ThriftMessageType::Call);

        let (res, future) = Future::<(ThriftMessage, BinaryDeserializer<Cursor<Vec<u8>>>)>::channel();
        client.send(Incoming::Call("foobar123".to_string(), buf, Some(res))).unwrap();

        let (res_tx, res_rx) = channel();
        let cloned = res_tx.clone();
        future.and_then(move |(msg, de)| {
            println!("[test]: Received: {:?}", msg);
            SENDER.clone().send(Message::Shutdown);
            res_tx.send(0);
            Async::Ok(())
        });

        // Ensure that the test exists after at least 3 seconds if the response was not
        // received.
        thread::spawn(move || -> Result<(), ()> {
            thread::sleep(Duration::from_millis(3000));
            SENDER.clone().send(Message::Shutdown);
            panic!("Test timeout was hit. This means the Reactor did not shutdown and a response was not received.");
            cloned.send(1);
        });

        Reactor::run().join();

        assert_eq!(res_rx.recv().unwrap(), 0);
    }
}
