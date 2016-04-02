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

#[derive(Debug)]
pub enum Role {
    /// A server will be tasked with actually calling a user defined
    /// RPC method and dispatching the response back to the event loop.
    Server(SocketAddr),
    /// A client is tasked with sending an initial RPC and dispatching a response.
    ///
    Client(SocketAddr)
}

pub enum Incoming {
    /// Method name, data buf, and response channel.
    Call(String, Vec<u8>, Sender<(ThriftMessage, BinaryDeserializer<Cursor<Vec<u8>>>)>),
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

            match role {
                Role::Server(addr) => {
                    event_loop_sender.send(Message::Bind(addr, id_tx, data_tx))?;
                },
                Role::Client(addr) => {
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
                            self.queue.insert(method, tx);
                        },
                        // The sender-part of the channel has been disconnected.
                        Err(err) => break
                    }
                },
                event_loop_msg = event_loop_rx.recv() => {
                    match event_loop_msg {
                        Ok(Dispatch::Data(token, buf)) => {
                            let mut de = BinaryDeserializer::new(Cursor::new(buf));
                            let msg = de.read_message_begin()?;

                            match msg.ty {
                                ThriftMessageType::Call => {
                                    if let Role::Client(_) = self.role {
                                        // A client isn't supposed to receive RPC calls.
                                    } else {
                                        // The server has received a call, so let's reply:
                                        let buf = util::create_empty_thrift_message("foobar123", ThriftMessageType::Reply);
                                        self.event_loop.send(Message::Rpc(token, buf));
                                    }
                                },
                                ThriftMessageType::Reply => {
                                    if let Role::Server(_) = self.role {
                                        // Servers never get a reply message. Ignore.
                                    } else {
                                        // Look into the request cache.
                                        match self.queue.remove(&msg.name) {
                                            Some(tx) => {
                                                tx.send((msg, de))?;
                                            },
                                            None => {}
                                        }
                                    }
                                },
                                _ => {}
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
    use util;

    #[test]
    fn should_create_server_dispatcher() {
        let addr = "127.0.0.1:5495".parse().unwrap();
        let (handle, tx) = Dispatcher::spawn(Role::Server(addr)).unwrap();
    }

    #[test]
    fn should_start_server() {
        let addr: SocketAddr = "127.0.0.1:5955".parse().unwrap();
        let (handle_server, server) = Dispatcher::spawn(Role::Server(addr.clone())).unwrap();
        let (handle_client, client) = Dispatcher::spawn(Role::Client(addr.clone())).unwrap();

        let buf = util::create_empty_thrift_message("foobar123", ThriftMessageType::Call);

        let (res, future) = Future::<(ThriftMessage, BinaryDeserializer<Cursor<Vec<u8>>>)>::channel();
        client.send(Incoming::Call("foobar123".to_string(), buf, res)).unwrap();

        future.and_then(move |(msg, de)| {
            println!("[test]: Received: {:?}", msg);
            SENDER.clone().send(Message::Shutdown);
            Async::Ok(())
        });

        Reactor::run().join();
    }
}
