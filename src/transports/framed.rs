use transport::Transport;
use std::sync::mpsc::{Receiver, Sender, channel};
use dispatcher::{Message, dispatch};
use std::net::SocketAddr;

/// The FramedTransport is required for async I/O and is
/// built around the `Reactor` and `Dispatcher`.
pub struct FramedTransport {
    dispatch: Sender<Message>
}

impl FramedTransport {
    pub fn new(addr: SocketAddr) -> FramedTransport {
        FramedTransport {
            dispatch: dispatch(addr)
        }
    }
}
