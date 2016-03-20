use mio::tcp::*;
use mio;
use mio::{Token, Handler, EventLoop, EventSet, PollOpt, TryRead, TryWrite, Evented};
use slab::Slab;
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

pub enum Message {
    InitServer(net::TcpListener, SocketAddr, Sender<Dispatch>),
    Rpc(Vec<u8>),
    Shutdown,
    Close
}

pub enum Dispatch {
    Data(Vec<u8>)
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
            // XXX
            self.chan.send(Dispatch::Data(buf));
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

/// XXX: Generalize the listener. Allow multiple listeners
/// and multiple connections so we can multiplex a bunch
/// of Thrift services in one `EventLoop`.
pub struct Reactor {
    listeners: HashMap<Token, TcpListener>,
    connections: HashMap<Token, Connection>,
    sender: Sender<Dispatch>,
    servers: Vec<Sender<Dispatch>>,
    current_token: usize
}

impl Reactor {
    pub fn new(sender: Sender<Dispatch>) -> Self {
        Reactor {
            listeners: HashMap::new(),
            connections: HashMap::new(),
            sender: sender,
            servers: Vec::new(),
            current_token: 0
        }
    }

    pub fn run(&mut self) -> ThrustResult<(EventLoop<Self>, mio::Sender<Message>)> {
        let mut event_loop = try!(EventLoop::new());

        // let mut buf = mem::replace(&mut self.buf, vec![]);
        // for stream in buf.into_iter() {
        //     let clone = self.sender.clone();
        //     let token = self.connections.insert_with(|token| {
        //         Connection::new(stream, token, clone)
        //     }).expect("Failed to insert a new connection in the slab");

        //     self.connections[token].register(&mut event_loop, token);
        // }

        let sender = event_loop.channel();
        Ok((event_loop, sender))
    }

    pub fn accept_connection(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
        let mut listener = self.listeners.get_mut(&token).unwrap();
        match listener.accept() {
            Ok(Some(socket)) => {
                let (stream, _) = socket;
                let clone = self.sender.clone();
                let new_token = Token(self.current_token);
                let mut conn = Connection::new(stream, new_token, clone);
                conn.register(event_loop, new_token);

                self.connections.insert(new_token, conn);
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
        if events.is_error() {
            println!("Err");
            return;
        }

        if events.is_hup() {
            return;
        }

        if events.is_readable() && self.listeners.contains_key(&token) {
            self.accept_connection(event_loop, token);
            return;
        }

        self.connections.get_mut(&token).unwrap().ready(event_loop, events);
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Timeout) {
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Message) {
        match msg {
            Message::Rpc(data) => {
                self.connections.get_mut(&Token(1)).unwrap().write(&*data);
            },
            Message::Shutdown => {
                event_loop.shutdown();
            },
            Message::InitServer(listener, addr, tx) => {
                let mut lis = TcpListener::from_listener(listener, &addr).unwrap();
                self.servers.push(tx);

                event_loop.register(&lis, Token(self.current_token), EventSet::readable(), PollOpt::edge()).unwrap();
                self.current_token += 1;
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use tangle::{Future, Async};
    use std::thread;
}
