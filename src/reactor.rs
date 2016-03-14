use mio::tcp::*;
use mio;
use mio::{Token, Handler, EventLoop, EventSet, PollOpt, TryRead, TryWrite, Evented};
use slab::Slab;
use std::io::{self, Cursor, Write, Read};
use std::net::SocketAddr;
use std::time::Duration;
use std::mem;
use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};
use result::{ThrustResult, ThrustError};
use tangle::{Future, Async};
use bytes::buf::Buf;

pub enum Message {
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

pub struct Reactor {
    listener: Option<TcpListener>,
    connections: Slab<Connection, Token>,
    buf: Vec<TcpStream>,
    sender: Sender<Dispatch>
}

impl Reactor {
    pub fn new(sender: Sender<Dispatch>) -> Self {
        Reactor {
            listener: None,
            connections: Slab::new_starting_at(Token(1), 1024),
            buf: Vec::new(),
            sender: sender
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) -> ThrustResult<()> {
        let mut stream = try!(TcpStream::connect(&addr));

        self.buf.push(stream);
        Ok(())
    }

    pub fn server(mut self, addr: SocketAddr) -> ThrustResult<Self> {
        let mut listener = try!(TcpListener::bind(&addr));

        self.listener = Some(listener);
        Ok(self)
    }

    pub fn run(&mut self) -> ThrustResult<(EventLoop<Self>, mio::Sender<Message>)> {
        let mut event_loop = try!(EventLoop::new());

        if let &Some(ref listener) = &self.listener {
            try!(event_loop.register(listener, Token(0), EventSet::readable(),
                            PollOpt::edge()));
        }

        let mut buf = mem::replace(&mut self.buf, vec![]);
        for stream in buf.into_iter() {
            let clone = self.sender.clone();
            let token = self.connections.insert_with(|token| {
                Connection::new(stream, token, clone)
            }).expect("Failed to insert a new connection in the slab");

            self.connections[token].register(&mut event_loop, token);
        }

        let sender = event_loop.channel();
        Ok((event_loop, sender))
    }

    pub fn accept_connection(&mut self, event_loop: &mut EventLoop<Self>) {
        if let Some(ref listener) = self.listener {
            match listener.accept() {
                Ok(Some(socket)) => {
                    let (stream, _) = socket;
                    let clone = self.sender.clone();
                    let token = self.connections.insert_with(|token| {
                        Connection::new(stream, token, clone)
                    }).expect("Failed to insert a new connection in the slab");

                    self.connections[token].register(event_loop, token);
                },
                _ => {}
            }
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

        if events.is_readable() && Token(0) == token {
            self.accept_connection(event_loop);
            return;
        }

        self.connections[token].ready(event_loop, events);
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Timeout) {
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Message) {
        match msg {
            Message::Rpc(data) => {
                self.connections[Token(1)].write(&*data);
            },
            Message::Shutdown => {
                event_loop.shutdown();
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

    #[test]
    fn create_reactor() {
        let (tx, rx) = channel();
        let addr = "127.0.0.1:5566".parse().unwrap();
        let mut reactor = Reactor::new(tx).server(addr).unwrap();

        let (mut event_loop, _) = reactor.run().expect("Error trying to run the Reactor");
        // event_loop.run(&mut reactor);
    }

    #[test]
    fn connect_client() {
        thread::spawn(move || {
            let (tx, rx) = channel();
            let addr = "127.0.0.1:5567".parse().unwrap();
            let mut reactor = Reactor::new(tx).server(addr).unwrap();

            let (mut event_loop, sender) = reactor.run().expect("Error trying to run the Reactor");

            thread::spawn(move || {
                for msg in rx.iter() {
                    match msg {
                        Dispatch::Data(buf) => {
                            assert_eq!(&*buf, &[1, 2, 3]);
                            sender.send(Message::Shutdown);
                        }
                    }
                }
            });

            event_loop.run(&mut reactor);
        });

        let (tx, rx) = channel();
        let addr = "127.0.0.1:5567".parse().unwrap();
        let mut reactor = Reactor::new(tx);

        reactor.connect(addr).unwrap();

        let (mut event_loop, sender) = reactor.run().expect("Error trying to run the Reactor");

        thread::spawn(move || {
            sender.send(Message::Rpc(vec![1, 2, 3]));
            sender.send(Message::Shutdown);
        });

        event_loop.run(&mut reactor);
    }
}
