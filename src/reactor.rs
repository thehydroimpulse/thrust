use mio::tcp::*;
use mio;
use mio::{Token, Handler, EventLoop, EventSet, PollOpt, TryRead, TryWrite, Evented};
use std::io::{self, Cursor, Write, Read};
use std::net::{self, SocketAddr};
use std::time::Duration;
use std::mem;
use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};
use result::{ThrustResult, ThrustError};
use tangle::{Future, Async};
use bytes::buf::Buf;
use std::collections::HashMap;

/// Communication into the Mio event loop happens with a `Message`. For each new Mio
/// event loop, a mio-specific `Sender<Message>` is returned.
pub enum Message {
    /// `Connect` establishes a new `TcpStream` with a specified remote. The
    /// `Sender` channel part is used to communicate back with the initiator on
    /// certain socket events.
    ///
    /// XXX: We should provide a way to accept a blocking `net::TcpStream` and convert
    /// it into a non-blocking mio `TcpStream`.
    Connect(SocketAddr, Sender<Dispatch>),
    /// To give a tighter feedback loop, a `Bind` message will accept a normal
    /// Rust blocking net::TcpListener. This allows the user to more easily handle
    /// binding errors before sending it into the event loop where you need to
    /// handle any errors asynchronously.
    Bind(net::TcpListener, SocketAddr, Sender<Dispatch>),
    /// Initiate an `Rpc` request. Each request needs to know which `Token` the respective
    /// `Connection` is associated with. The `Reactor` also knows nothing about Thrift
    /// and simply works at the binary level.
    ///
    /// An `Rpc` message is also used for replying to an RPC call.
    Rpc(Token, Vec<u8>),
    /// Completely shutdown the `Reactor` and event loop. All current listeners
    /// and connections will be dropped.
    Shutdown
}

/// Communication from the `Reactor` to outside components happens with a `Dispatch` message
/// and normal Rust channels instead of Mio's variant.
pub enum Dispatch {
    /// Each connection and listener is tagged with a Mio `Token` so we can differentiate between
    /// them. As soon as we create the respective resource, we need a `Dispatch::Id` message
    /// containing the newly allocated `Token`.
    ///
    /// This is used to send RPC calls or further differentiate each resource outside the
    /// event loops.
    Id(Token),
    /// When a socket has been read, the `Reactor` will send the `Dispatch::Data` message
    /// to the associating channel.
    ///
    /// We also associate any incoming data with the Token of the responsible socket.
    Data(Token, Vec<u8>)
}

pub enum Timeout {
    Reconnect(Token, SocketAddr)
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Reading,
    Writing,
    Closed
}

pub struct Connection {
    stream: TcpStream,
    pub token: Token,
    state: State,
    chan: Sender<Dispatch>,
    rbuffer: Vec<u8>,
    wbuffer: Cursor<Vec<u8>>
}

impl Connection {
    pub fn new(stream: TcpStream, token: Token, chan: Sender<Dispatch>) -> Self {
        Connection {
            stream: stream,
            token: token,
            state: State::Reading,
            chan: chan,
            rbuffer: vec![],
            wbuffer: Cursor::new(vec![])
        }
    }

    pub fn ready(&mut self, event_loop: &mut EventLoop<Reactor>, events: EventSet) {
        match self.state {
            State::Reading if events.is_readable() => {
                self.readable();
                self.reregister(event_loop, self.token);
            },
            State::Writing if events.is_writable() => {
                self.writable();
                self.reregister(event_loop, self.token);
            },
            _ => {
                self.reregister(event_loop, self.token);
            }
        }
    }

    pub fn read(&mut self) -> ThrustResult<Vec<u8>> {
        match self.stream.try_read_buf(&mut self.rbuffer) {
            Ok(Some(_)) => Ok(mem::replace(&mut self.rbuffer, vec![])),
            Ok(None) => Err(ThrustError::NotReady),
            Err(err) => Err(ThrustError::Other)
        }
    }

    pub fn writable(&mut self) -> ThrustResult<()> {
        // Flush the whole buffer. The socket can, at any time, be unwritable. Thus, we
        // need to keep track of what we've written so far.
        while self.wbuffer.has_remaining() {
            self.flush();
        }

        self.state = State::Reading;

        Ok(())
    }

    pub fn readable(&mut self) -> ThrustResult<()> {
        while let Ok(buf) = self.read() {
            self.chan.send(Dispatch::Data(self.token, buf));
        }

        self.state = State::Writing;

        Ok(())
    }

    fn register(&mut self, event_loop: &mut EventLoop<Reactor>, token: Token) -> ThrustResult<()> {
        try!(event_loop.register(&self.stream, token, EventSet::readable(),
                            PollOpt::edge() | PollOpt::oneshot()));
        Ok(())
    }

    pub fn reregister(&self, event_loop: &mut EventLoop<Reactor>, token: Token) -> ThrustResult<()> {
        let event_set = match self.state {
            State::Reading => EventSet::readable(),
            State::Writing => EventSet::writable(),
            _ => EventSet::none()
        };

        try!(event_loop.reregister(&self.stream, self.token, event_set, PollOpt::oneshot()));
        Ok(())
    }
}

impl Write for Connection {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        try!(self.wbuffer.get_mut().write(data));
        try!(self.flush());
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.stream.try_write_buf(&mut self.wbuffer) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Ok(()),
            Err(err) => Err(err)
        }
    }
}

/// The `Reactor` is the component that interacts with networking. The reactor is
/// built around Mio's event loop and manages both TcpListeners and TcpStreams.
///
/// The reactor isn't responsible for anything Thrift related, so it doesn't know
/// about parsing, protocols, serialization, etc... All it's responsible for
/// is sending and receiving data from various connections and dispatching them
/// to the appropriate channels.
///
/// To communicate into the event loop, you use a copy of Mio's `Sender` channel
/// type to send `Message`s. These will be intercepted in the event loop and processed.
///
/// Things you might send to the `Reactor` through this mechanism:
///
/// 1. Binding a new TCP listener &mdash; Each reactor is capable of handling an unbounded
/// number of listeners, who will all be able to accept new sockets.
///
/// Binding a new listener requires that you have already established a blocking variant
/// through `net::TcpListener`. The listener will be converted to Mio's non-blocking variant.
///
/// ```notrust
/// use std::net::TcpListener;
/// use std::sync::mpsc::channel;
///
/// // The channel is used as a channel. Any socket being accepted
/// // by this listener will also use this channel (the sender part).
/// let (tx, rx) = channel();
/// let listener = TcpListener::bind("127.0.0.1:4566").unwrap();
/// let addr = "127.0.0.1:4566".parse().unwrap();
///
/// reactor_sender.send(Message::Bind(listener, addr, tx));
/// ```
///
/// 2. Connecting to a remote TCP server and establishing a new non-blocking `TcpStream`.
///
/// ```notrust
/// use std::sync::mpsc::channel;
///
/// // The callback channel on the single socket.
/// let (tx, rx) = channel();
/// let addr = "127.0.0.1::4566".parse().unwrap();
/// reactor_sender.send(Message::Connect(addr, tx));
/// ```
///
///
/// 3. Sending RPC calls (initiating or reply) to a socket. Instead of writing or reading
/// primitives, we boil everything down to Rpc or Data messages, each with an associative Token
/// to mark the responsible `TcpStream` or `Connection`.
///
/// ```notrust
/// reactor_sender.send(Message::Rpc(Token(1), vec![0, 1, 3, 4]));
/// ```
pub struct Reactor {
    listeners: HashMap<Token, TcpListener>,
    connections: HashMap<Token, Connection>,
    /// Channels that are sent from `::Bind` messages to establish a listener will
    /// be appended in this map. All subsequent sockets being accepted from the listener
    /// will use the same sender channel to consolidate communications.
    servers: HashMap<Token, Sender<Dispatch>>,
    /// The `Reactor` manages a count of the number of tokens, the number being
    /// the token used for the next allocated resource. Tokens are used sequentially
    /// across both listeners and connections.
    current_token: usize
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            listeners: HashMap::new(),
            connections: HashMap::new(),
            servers: HashMap::new(),
            current_token: 0
        }
    }

    pub fn accept_connection(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
        let mut listener = self.listeners.get_mut(&token).expect("Listener was not found.");
        match listener.accept() {
            Ok(Some(socket)) => {
                let (stream, _) = socket;
                let clone = self.servers[&token].clone();
                let new_token = Token(self.current_token);
                let mut conn = Connection::new(stream, new_token, clone);

                self.connections.insert(new_token, conn);
                self.connections.get_mut(&new_token)
                    .unwrap()
                    .register(event_loop, new_token);

                self.current_token += 1;
            },
            _ => {}
        }
    }
}

impl Handler for Reactor {
    type Timeout = Timeout;
    type Message = Message;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        if events.is_hup() {
            println!("Hup received. Socket disconnected.");
            if self.connections.contains_key(&token) {
                self.connections.remove(&token);
            }
            return;
        }

        if events.is_error() {
            println!("Err: {:?}", events);
            return;
        }

        if events.is_readable() && self.listeners.contains_key(&token) {
            self.accept_connection(event_loop, token);
            return;
        }

        if self.connections.contains_key(&token) {
            self.connections.get_mut(&token).expect("connection was not found.").ready(event_loop, events);
        }
    }

    /// XXX: Timeouts would be useful to implement.
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Timeout) {
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Message) {
        match msg {
            Message::Rpc(id, data) => {
                self.connections.get_mut(&id).expect("connection was not found.").write(&*data);
            },
            Message::Shutdown => {
                event_loop.shutdown();
            },
            Message::Connect(addr, tx) => {
                let mut mio_stream = TcpStream::connect(&addr).expect("MIO ERR");
                let new_token = Token(self.current_token);
                tx.send(Dispatch::Id(new_token));
                let mut conn = Connection::new(mio_stream, new_token, tx);

                self.connections.insert(new_token, conn);

                self.connections.get_mut(&new_token)
                    .unwrap()
                    .register(event_loop, new_token);

                self.current_token += 1;
            },
            Message::Bind(listener, addr, tx) => {
                let token = Token(self.current_token);
                let mut lis = TcpListener::from_listener(listener, &addr).unwrap();
                tx.send(Dispatch::Id(token));
                self.servers.insert(token, tx);

                event_loop.register(&lis, token, EventSet::readable(), PollOpt::edge()).unwrap();
                self.listeners.insert(token, lis);
                self.current_token += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mio::{EventLoop, Token};
    use super::*;
    use std::io::Write;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use tangle::{Future, Async};
    use std::thread;
    use std::time::Duration;
    use std::net::{TcpListener, TcpStream, SocketAddr};

    #[test]
    fn create_reactor() {
        let (assert_tx, assert_rx) = channel();
        let mut reactor = Reactor::new();
        let mut event_loop = EventLoop::new().unwrap();
        let sender = event_loop.channel();

        let handle = thread::spawn(move || {
            event_loop.run(&mut reactor);
        });

        // Establish a local TcpListener.
        let addr: SocketAddr = "127.0.0.1:5543".parse().unwrap();
        let listener = TcpListener::bind(addr.clone()).unwrap();
        let (rpc_server_tx, rpc_server_rx) = channel();

        // Create a new non-blocking tcp server.
        sender.send(Message::Bind(listener, addr.clone(), rpc_server_tx.clone()));

        let (rpc_client_tx, rpc_client_rx) = channel();

        sender.send(Message::Connect(addr, rpc_client_tx));

        let client_id = match rpc_client_rx.recv().unwrap() {
            Dispatch::Id(n) => n,
            _ => panic!("Expected to receive the Connection id/token.")
        };

        sender.send(Message::Rpc(client_id, b"abc".to_vec()));

        let server = thread::spawn(move || {
            for msg in rpc_server_rx.iter() {
                match msg {
                    Dispatch::Data(id, msg) => {
                        assert_tx.send((id, msg)).expect("Could not assert_tx");
                    },
                    _ => {}
                }
            }
        });

        let (new_id, v) = assert_rx.recv().expect("Error trying to assert reactor test.");
        assert_eq!(new_id, Token(2));
        assert_eq!(v.len(), 3);
        assert_eq!(v, b"abc");

        // Send a "response" back.
        sender.send(Message::Rpc(new_id, b"bbb".to_vec()));


        match rpc_client_rx.recv().unwrap() {
            Dispatch::Data(id, v) => {
                assert_eq!(id, client_id);
                assert_eq!(v, b"bbb");
            },
            _ => panic!("Unexpected case.")
        }
    }
}
