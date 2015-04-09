use transport::Transport;
use std::net::{TcpListener, TcpStream};

enum Kind {
    Stream(TcpStream),
    Server(TcpListener)
}

pub struct TcpTransport {
    addr: String,
    /// A `TcpTransport` can work with either a single `TcpStream`
    /// or with a `TcpListener`.
    kind: Kind
}
