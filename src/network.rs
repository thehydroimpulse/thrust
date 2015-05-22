use nom::{Producer, ProducerState};
use std::net::TcpStream;
use std::io::{Write, Read, SeekFrom};
use std::iter::repeat;

pub struct NetworkProducer {
    size: usize,
    stream: TcpStream,
    v: Vec<u8>
}

impl NetworkProducer {
    pub fn new(stream: TcpStream, buffer_size: usize) -> NetworkProducer {
        NetworkProducer {
            size: buffer_size,
            stream: stream,
            v: Vec::with_capacity(buffer_size)
        }
    }
}

impl Producer for NetworkProducer {
    fn produce(&mut self) -> ProducerState<&[u8]> {
        let len = self.v.len();

        self.v.extend(repeat(0).take(self.size - len));

        match self.stream.read(&mut self.v) {
            Ok(n) => {
                self.v.truncate(n);
                if n == 0 {
                    ProducerState::Eof(&self.v[..])
                } else {
                    ProducerState::Data(&self.v[..])
                }
            },
            Err(e) => ProducerState::ProducerError(0)
        }
    }

    fn seek(&mut self, position: SeekFrom) -> Option<u64> {
        Some(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread::spawn;
    use std::io::{Read, Write};
    use nom::{Producer, ProducerState};

    #[test]
    fn simple() {
        spawn(move || {
            let mut listener = TcpListener::bind("localhost:6767").unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        stream.write(b"123");
                    },
                    Err(_) => {}
                }
            }
        });

        let mut conn = TcpStream::connect("localhost:6767").unwrap();
        let mut producer = NetworkProducer::new(conn, 8);
        assert_eq!(producer.produce(), ProducerState::Data(&b"123"[..]));
    }
}
