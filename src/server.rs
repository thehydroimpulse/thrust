use runner::Runner;
use protocol::{Deserializer, ThriftDeserializer};
use binary_protocol::BinaryDeserializer;
use std::io::Cursor;
use spawner::Spawner;
use reactor::{Reactor, Message, Dispatch};
use std::thread;
use mio;
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

impl<R> Server<R>
    where R: 'static + Runner<Default> + Send
{
    pub fn new(spawner: &Spawner, runner: R) -> Server<R> {
        Server {
            senders: spawner.get_senders(),
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
            for sender in self.senders.iter() {
                // Bind to a blocking `TcpListener` to get instant feedback
                // on if it connected successfully or threw an error. It will
                // be converted to a non-blocking socket before it's used.
                let mut listener = TcpListener::bind(addr).unwrap();
                sender.send(Message::Bind(listener, addr, tx.clone()));
            }

            for msg in rx.iter() {
                println!("Server received a dispatch from Mio.");
            }
        });
    }
}
