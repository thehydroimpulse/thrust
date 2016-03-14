use num_cpus;
use std::sync::mpsc::{Receiver, channel};
use mio::{Sender, EventLoop};
use reactor::{Reactor, Message, Dispatch};
use std::thread::{self, JoinHandle};

pub struct Spawner {
    handles: Vec<JoinHandle<()>>,
    senders: Vec<Sender<Message>>
}

impl Spawner {
    pub fn new(cpus: Option<usize>) -> Spawner {
        let num = cpus.unwrap_or_else(|| num_cpus::get());
        let mut handles = Vec::with_capacity(num);
        let mut senders = Vec::with_capacity(num);

        for i in 0..num {
            let (tx, rx) = channel();
            let mut event_loop = EventLoop::new().unwrap();

            senders.push(event_loop.channel());

            handles.push(thread::spawn(move || {
                let mut reactor = Reactor::new(tx);
                event_loop.run(&mut reactor);
            }));
        }

        Spawner {
            handles: handles,
            senders: senders
        }
    }

    pub fn (mut self) {
        for handle in 0..self.handles.len() {
            handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn() {
        let spawner = Spawner::new(Some(2));
        assert_eq!(spawner.handles.len(), 2);
        assert_eq!(spawner.senders.len(), 2);
    }
}
