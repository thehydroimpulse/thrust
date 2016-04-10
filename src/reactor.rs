use mio::tcp::*;
use mio;
use mio::{Token, Handler, EventLoop, EventSet, PollOpt, TryRead, TryWrite, Evented};
use std::io::{self, Cursor, Write, Read};
use std::net::{self, SocketAddr};
use std::time::Duration;
use std::mem;
use std::iter;
use std::thread::{self, JoinHandle};
use event_loop::EVENT_LOOP;
use std::sync::mpsc::{Receiver, Sender, channel};
use result::{ThrustResult, ThrustError};
use tangle::{Future, Async};
use bytes::buf::Buf;
use std::collections::HashMap;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libc;
use std::os::unix::io::AsRawFd;

pub struct Id(pub Token);

/// Communication into the Mio event loop happens with a `Message`. For each new Mio
/// event loop, a mio-specific `Sender<Message>` is returned.
#[derive(Debug, Clone)]
pub enum Message {
    /// `Connect` establishes a new `TcpStream` with a specified remote. The
    /// `Sender` channel part is used to communicate back with the initiator on
    /// certain socket events.
    ///
    /// The first `Sender` is used to communicate back the assigned `Token`.
    Connect(SocketAddr, Sender<Id>, Sender<Dispatch>),
    /// To give a tighter feedback loop, a `Bind` message will accept a normal
    /// Rust blocking net::TcpListener. This allows the user to more easily handle
    /// binding errors before sending it into the event loop where you need to
    /// handle any errors asynchronously.
    Bind(SocketAddr, Sender<Id>, Sender<Dispatch>),
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
#[derive(Debug)]
pub enum Dispatch {
    /// When a socket has been read, the `Reactor` will send the `Dispatch::Data` message
    /// to the associating channel.
    ///
    /// We also associate any incoming data with the Token of the responsible socket.
    Data(Token, Vec<u8>)
}

pub enum Timeout {
    Reconnect(Token)
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// The length that has been read so far.
    ReadingFrame(usize),
    Reading,
    Writing,
    Closed
}

#[derive(Debug)]
pub enum FrameState {
    Reading(u32),
    Writing
}

/// Wrap a `TcpStream` to handle reading and writing frames. Frames are simply some encoded thrift
/// protocol byte buffer preceeded by a 32-bit unsigned length.
pub struct FramedTransport {
    buffer: Vec<u8>,
    state: FrameState
}

impl FramedTransport {
    pub fn new() -> FramedTransport {
        FramedTransport {
            buffer: Vec::new(),
            state: FrameState::Reading(0)
        }
    }

    pub fn read<S>(&mut self, socket: &mut S) -> ThrustResult<Option<Vec<u8>>>
        where S: TryRead + TryWrite
    {
        match self.state {
            FrameState::Reading(total_len) if self.buffer.len() == 0 => {
                let mut buf = Vec::with_capacity(4);
                buf.extend(iter::repeat(0).take(4));

                // Try reading the first unsigned 32-bits for the length of the frame.
                let len = match socket.try_read_buf(&mut buf)? {
                    Some(n) if n == 4 => {
                        let mut buf = Cursor::new(buf);
                        buf.read_u32::<BigEndian>()?
                    },
                    Some(n) => {
                        return Err(ThrustError::Other);
                    },
                    None => panic!("err")
                };

                // Set our internal buffer.
                self.buffer = Vec::with_capacity(len as usize);
                self.buffer.extend(iter::repeat(0).take(len as usize));
                return self.read(socket);
            },
            FrameState::Reading(ref mut total_len) => {
                while let Some(n) = socket.try_read_buf(&mut self.buffer)? {
                    *total_len += n as u32;
                    if *total_len == self.buffer.len() as u32 {
                        return Ok(Some(mem::replace(&mut self.buffer, Vec::new())));
                    } else if n == 0 {
                        return Ok(None);
                    }
                }
            },
            FrameState::Writing => return Err(ThrustError::Other)
        }

        Ok(None)
    }
}


pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    pub token: Token,
    state: State,
    chan: Sender<Dispatch>,
    rbuffer: Vec<u8>,
    wbuffer: Cursor<Vec<u8>>
}

impl Connection {
    pub fn new(conn: (TcpStream, SocketAddr), token: Token, chan: Sender<Dispatch>) -> Self {
        Connection {
            stream: conn.0,
            addr: conn.1,
            token: token,
            state: State::Reading,
            chan: chan,
            rbuffer: vec![],
            wbuffer: Cursor::new(vec![])
        }
    }

    pub fn reset(&mut self, event_loop: &mut EventLoop<Reactor>) {
        event_loop.timeout(Timeout::Reconnect(self.token), Duration::from_millis(10));
    }

    pub fn ready(&mut self, event_loop: &mut EventLoop<Reactor>, events: EventSet) {
        match self.state {
            State::Reading if events.is_readable() => {
                match self.readable() {
                    Ok(_) => {},
                    Err(err) => {
                        println!("[reactor]: could not dispatch incoming data. {:?}", err);
                        panic!("{:?}", err);
                    }
                }
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

    pub fn read(&mut self) -> ThrustResult<Option<Vec<u8>>> {
        match self.state {
            State::Reading => {
                let len = self.stream.read_u32::<BigEndian>()?;
                self.state = State::ReadingFrame(0);
                self.rbuffer = Vec::with_capacity(len as usize);
                self.read()
            },
            State::ReadingFrame(ref mut len) => {
                match self.stream.try_read_buf(&mut self.rbuffer) {
                    Ok(Some(n)) => {
                        if *len + n == self.rbuffer.len() {
                            let buf = mem::replace(&mut self.rbuffer, Vec::new());
                            Ok(Some(buf))
                        } else {
                            *len += n;
                            // We don't have a complete frame yet.
                            Ok(None)
                        }
                    },
                    Ok(None) => Err(ThrustError::NotReady),
                    Err(err) => Err(ThrustError::Other)
                }
            },
            _ => Err(ThrustError::Other)
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
        while let Ok(op) = self.read() {
            match op {
                Some(buf) => {
                    self.state = State::Reading;
                    println!("[reactor/connection]: reading data from {:?}", self.token);
                    try!(self.chan.send(Dispatch::Data(self.token, buf)));
                },
                None => {}
            }
        }

        self.state = State::Writing;

        Ok(())
    }

    fn register(&mut self, event_loop: &mut EventLoop<Reactor>, token: Token) -> ThrustResult<()> {
        event_loop.register(&self.stream, token, EventSet::readable(), PollOpt::edge() | PollOpt::oneshot())?;
        Ok(())
    }

    pub fn reregister(&self, event_loop: &mut EventLoop<Reactor>, token: Token) -> ThrustResult<()> {
        let event_set = match self.state {
            State::Reading => EventSet::readable(),
            State::Writing => EventSet::writable(),
            _ => EventSet::none()
        };

        event_loop.reregister(&self.stream, self.token, event_set, PollOpt::oneshot())?;
        Ok(())
    }
}

impl Write for Connection {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.wbuffer.get_mut().write_u32::<BigEndian>(data.len() as u32)?;
        self.wbuffer.get_mut().write(data)?;
        self.flush()?;
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
/// let addr = "127.0.0.1:4566".parse().unwrap();
///
/// reactor_sender.send(Message::Bind(addr, tx));
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
    pub fn new() -> Reactor {
        Reactor {
            listeners: HashMap::new(),
            connections: HashMap::new(),
            servers: HashMap::new(),
            current_token: 0
        }
    }

    pub fn run() -> JoinHandle<()> {
        thread::spawn(move || {
            let mut event_loop = EVENT_LOOP.lock().expect("Failed to take the `EVENT_LOOP` lock.");
            let mut reactor = Reactor::new();
            event_loop.run(&mut reactor);
        })
    }

    pub fn incoming_timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Timeout) -> ThrustResult<()> {
        match timeout {
            Timeout::Reconnect(token) => {
                let mut conn = self.connections.get_mut(&token).expect("Could not find the connection.");
                let mut stream = TcpStream::connect(&conn.addr)?;
                conn.stream = stream;
                conn.register(event_loop, token);
            }
        }

        Ok(())
    }

    pub fn incoming_msg(&mut self, event_loop: &mut EventLoop<Self>, msg: Message) -> ThrustResult<()> {
        match msg {
            Message::Rpc(id, data) => {
                println!("[reactor]: rpc @ {:?}", id);
                self.connections.get_mut(&id).expect("connection was not found #2").write(&*data);
            },
            Message::Shutdown => {
                println!("Shutting down...");
                event_loop.shutdown();
            },
            Message::Connect(addr, id_tx, tx) => {
                let mut mio_stream = TcpStream::connect(&addr)?;
                let new_token = Token(self.current_token);
                id_tx.send(Id(new_token));
                let mut conn = Connection::new((mio_stream, addr), new_token, tx);

                println!("[reactor]: binding to {:?} @ {:?}", addr, new_token);

                self.connections.insert(new_token, conn);
                self.connections.get_mut(&new_token)
                    .expect("Cannot find the connection from the token {:?}")
                    .register(event_loop, new_token);

                self.current_token += 1;
            },
            Message::Bind(addr, id_tx, tx) => {
                let token = Token(self.current_token);
                let mut lis = TcpListener::bind(&addr)?;

                id_tx.send(Id(token));
                self.servers.insert(token, tx);

                println!("[reactor]: binding to {:?} @ {:?}", addr, token);

                event_loop.register(&lis, token, EventSet::readable(), PollOpt::edge())?;
                self.listeners.insert(token, lis);
                self.current_token += 1;
            }
        }

        Ok(())
    }

    pub fn accept_connection(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
        let mut listener = self.listeners.get_mut(&token).expect("Listener was not found.");
        match listener.accept() {
            Ok(Some(socket)) => {
                let clone = self.servers[&token].clone();
                let new_token = Token(self.current_token);
                let mut conn = Connection::new(socket, new_token, clone);

                self.connections.insert(new_token, conn);
                self.connections.get_mut(&new_token)
                    .expect("Cannot find the connection in the cache.")
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
            if self.connections.contains_key(&token) {
                let mut conn = self.connections.get_mut(&token)
                    .expect("Cannot find the connection in the cache.")
                    .reset(event_loop);
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
            self.connections.get_mut(&token).expect("connection was not found #1").ready(event_loop, events);
        }
    }

    /// XXX: Timeouts would be useful to implement.
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Timeout) {
        match self.incoming_timeout(event_loop, timeout) {
            Ok(_) => {},
            Err(err) => panic!("An error occurred while handling a timeout")
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Message) {
        match self.incoming_msg(event_loop, msg) {
            Ok(_) => {},
            Err(err) => panic!("Reactor failed to handle incoming msg")
        }
    }
}

#[cfg(test)]
mod tests {
    use mio::{EventLoop, Token};
    use super::*;
    use std::io::{Write, Cursor, Read};
    use std::sync::mpsc::{Receiver, Sender, channel};
    use tangle::{Future, Async};
    use std::thread;
    use std::time::Duration;
    use std::net::{TcpListener, TcpStream, SocketAddr};
    use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

    // #[test]
    // fn should_read_frame() {
    //     let mut buf = vec![1, 2, 3];
    //     let mut source = Vec::new();
    //     source.write_u32::<BigEndian>(3);
    //     source.write(&mut buf);

    //     let mut source = Cursor::new(source);
    //     let mut framed = FramedTransport::new();
    //     let buf = match framed.read(&mut source) {
    //         Ok(Some(buf)) => buf,
    //         Ok(None) => panic!("Could not read the next frame from the socket."),
    //         Err(err) => panic!("Tests failed.")
    //     };

    //     assert_eq!(&buf[..], &[1, 2, 3]);
    // }

    // #[test]
    // fn should_error_on_incomplete_frame() {
    //     let mut buf = vec![1, 2];
    //     let mut source = Vec::new();
    //     source.write_u32::<BigEndian>(3);
    //     source.write(&mut buf);

    //     let mut source = Cursor::new(source);
    //     let mut framed = FramedTransport::new();
    //     match framed.read(&mut source) {
    //         Ok(Some(buf)) => panic!("We shouldn't have gotten a read frame back."),
    //         Ok(None) => {},
    //         Err(err) => panic!("Tests failed. {:?}", err)
    //     }
    // }

    // #[test]
    // fn should_eventually_read_delayed_frame() {
    //     let mut buf = vec![];
    //     let mut source = Vec::new();
    //     source.write_u32::<BigEndian>(3);
    //     source.write(&mut buf);

    //     let mut reader = Cursor::new(source);
    //     let mut framed = FramedTransport::new();
    //     match framed.read(&mut reader) {
    //         Ok(Some(buf)) => panic!("We shouldn't have gotten a read frame back."),
    //         Ok(None) => {},
    //         Err(err) => panic!("Tests failed. {:?}", err)
    //     }
    // }


    #[test]
    fn create_reactor() {
        let (assert_tx, assert_rx) = channel();
        let mut reactor = Reactor::new();
        let mut event_loop = EventLoop::new().expect("[test]: EventLoop failed to create.");
        let sender = event_loop.channel();

        let handle = thread::spawn(move || {
            event_loop.run(&mut reactor);
        });

        // Establish a local TcpListener.
        let addr: SocketAddr = "127.0.0.1:6543".parse().expect("[test]: Parsing into SocketAddr failed.");
        let (rpc_server_tx, rpc_server_rx) = channel();

        // Create a new non-blocking tcp server.
        let (id_tx, id_rx) = channel();
        sender.send(Message::Bind(addr.clone(), id_tx, rpc_server_tx.clone()));

        let (rpc_client_tx, rpc_client_rx) = channel();
        let (rpc_client_id_tx, rpc_client_id_rx) = channel();

        sender.send(Message::Connect(addr, rpc_client_id_tx, rpc_client_tx));

        let Id(client_id) = rpc_client_id_rx.recv().expect("[test]: Receiving from channel `rpc_client_id_rx` failed.");
        sender.send(Message::Rpc(client_id, b"abc".to_vec()));

        let server = thread::spawn(move || {
            for msg in rpc_server_rx.iter() {
                match msg {
                    Dispatch::Data(id, msg) => {
                        assert_tx.send((id, msg)).expect("Could not assert_tx");
                    }
                }
            }
        });

        let (new_id, v) = assert_rx.recv().expect("Error trying to assert reactor test.");
        assert_eq!(new_id, Token(2));
        assert_eq!(v.len(), 3);
        assert_eq!(v, b"abc");

        // Send a "response" back.
        sender.send(Message::Rpc(new_id, b"bbb".to_vec()));

        match rpc_client_rx.recv().expect("[test]: Receiving from channel `rpc_client_rx` failed.") {
            Dispatch::Data(id, v) => {
                assert_eq!(id, client_id);
                assert_eq!(v, b"bbb");
            }
        }
    }
}
