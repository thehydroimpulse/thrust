use std::io::{Read, Write};
use server::Task;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::net::{TcpStream, TcpListener};
use std::thread;
use threadpool::ThreadPool;

/// An abstraction over the Read and Write traits that are implemented
/// for socket/connection-like objects. The downside to this approach is
/// the need to manually implement `Stream` on the given connection types, however,
/// Rust doesn't support type aliases like `type Stream<T: Write + Read> = T;`.
pub trait Stream: Write + Read {}

impl Stream for TcpStream {}

/// Transport layer that deals with handling incoming and outgoing connections. This is the main
/// communication layer that touches the network. For server transports, outgoing communication
/// does **not** go through this transport layer. That's handled by the respective `Stream` of the
/// connection that should implement the `Write` trait.
pub trait Transport {
    /// Each transport can define it's own connection type that should
    /// implement both the `Read` and `Write` trait.
    type Connection: Stream;

    /// [server only]
    /// Called on an existing transport to start listening for new connections
    /// and accepting them. The address is passed along with a `Sender` part of
    /// a channel to communicate back with the server infrastructure.
    ///
    /// It's assumed that additional threads will be spawned, which is why the sender
    /// is passed as a parameter; however, the implementor of this method is responsible
    /// for spawning said threads.
    fn listen(&mut self, addr: &str, tx: Sender<Task<Self::Connection>>);
}

/// A thread pool backed, blocking TCP transport.
pub struct TcpTransport {
    pool: ThreadPool,
    streams: Vec<TcpStream>,
    acceptor_rx: Receiver<Task<TcpStream>>,
    acceptor_tx: Sender<Task<TcpStream>>
}

pub struct PoolSize(usize);

impl TcpTransport {
    pub fn new(pool_size: PoolSize) -> TcpTransport {
        let (tx, rx) = channel();

        let PoolSize(size) = pool_size;

        TcpTransport {
            pool: ThreadPool::new(size),
            streams: Vec::new(),
            acceptor_rx: rx,
            acceptor_tx: tx
        }
    }
}

impl Transport for TcpTransport {
    type Connection = TcpStream;

    fn listen(&mut self, addr: &str, tx: Sender<Task<TcpStream>>) {
        let transport_tx = self.acceptor_tx.clone();
        let addr = addr.to_string();

        thread::spawn(move || {
            let listener = TcpListener::bind(&*addr).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        tx.send(Task::IncomingStream(stream));
                    },
                    Err(err) => {}
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::mpsc::{channel, Sender, Receiver};
    use std::net::{TcpStream};
    use std::thread;
    use server::Task;

    #[test]
    fn tcp() {
        let (tx, rx) = channel();
        let mut transport = TcpTransport::new(PoolSize(4));
        transport.listen("localhost:5677", tx.clone());

        thread::spawn(move || {
            let mut stream = TcpStream::connect("localhost:5677").unwrap();
        }).join();

        let mut i = 0;

        match rx.recv().unwrap() {
            Task::IncomingStream(stream) => {
                i += 1;
            }
        }

        assert_eq!(i, 1);
    }
}
