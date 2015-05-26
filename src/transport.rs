use std::io::{Read, Write};
use server::Task;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::net::{TcpStream, TcpListener};
use std::thread;

pub trait Stream: Write + Read {}

impl Stream for TcpStream {}

pub trait Transport {
    type Connection: Stream;
    fn listen(&mut self, addr: &str, tx: Sender<Task<Self::Connection>>);
}

pub struct TcpTransport {
    streams: Vec<TcpStream>,
    acceptor_rx: Receiver<Task<TcpStream>>,
    acceptor_tx: Sender<Task<TcpStream>>
}

impl TcpTransport {
    pub fn new() -> TcpTransport {
        let (tx, rx) = channel();

        TcpTransport {
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
        let mut transport = TcpTransport::new();
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
