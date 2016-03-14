use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use event_loop::EVENT_LOOP;
use std::net::SocketAddr;
use request::Request;

pub enum Message {
    Req(Request, Sender<Vec<u8>>),
    Shutdown
}

pub struct RpcDispatch {
    rx: Receiver<Message>,
    queue: HashMap<u64, Sender<Vec<u8>>>
}

impl RpcDispatch {
    pub fn new(addr: SocketAddr, rx: Receiver<Message>) -> RpcDispatch {
        RpcDispatch {
            rx: rx,
            queue: HashMap::new()
        }
    }

    pub fn run(mut self) {
        for msg in self.rx.iter() {
            match msg {
                Message::Shutdown => break,
                Message::Req(req, tx) => {
                    tx.send(vec![123]);
                    self.queue.insert(req.id, tx);
                }
            }
        }
    }
}

pub fn dispatch(addr: SocketAddr) -> (JoinHandle<()>, Sender<Message>) {
    let (tx, rx) = channel();

    let handle = thread::spawn(move || {
        RpcDispatch::new(addr, rx).run();
    });

    (handle, tx)
}
