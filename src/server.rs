use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender, channel};
use transport::{Stream, Transport};

/// An opaque trait to implement against services.
pub trait Service {}

pub enum Task<S> where S: Stream {
    /// When the transport layer accepts a new stream/connection,
    /// they'll send a clone back to the server to store.
    IncomingStream(S)
}

/// Thrust server pieces together a transport, service, processor
/// and coordinates between them.
pub struct Server<'a, S, T> where T: Transport {
    service: S,
    addr: &'a str,
    transport: T,
    /// Receiver part of the channel that communicates
    /// with the transport layer.
    transport_rx: Receiver<Task<T::Connection>>,
    transport_tx: Sender<Task<T::Connection>>
}

impl<'a, S: Service, T: Transport> Server<'a, S, T> {
    pub fn new(addr: &'a str, service: S, transport: T) -> Server<'a, S, T> {
        let (tx, rx) = channel();

        Server {
            service: service,
            addr: addr,
            transport: transport,
            transport_rx: rx,
            transport_tx: tx
        }
    }

    /// Bind the Thrift service to a given transport.
    pub fn listen(&mut self) {
        let tx = self.transport_tx.clone();
        self.transport.listen(self.addr, tx);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use transport::{Stream, Transport};
    use std::sync::mpsc::Sender;
    use std::io::Cursor;

    struct FooService;
    impl Service for FooService {}

    struct FakeTransport;

    impl Stream for Cursor<Vec<u8>> {}

    impl Transport for FakeTransport {
        type Connection = Cursor<Vec<u8>>;
        fn listen(&mut self, addr: &str, tx: Sender<Task<Self::Connection>>) {
            assert_eq!(1, 1);
        }
    }

    #[test]
    fn new_server() {
        let mut server = Server::new("localhost:5966", FooService, FakeTransport);
        server.listen();
    }
}
