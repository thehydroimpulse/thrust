use mio::{EventLoop, Sender};
use reactor::{Reactor, Message};
use std::sync::{Arc, Mutex};
use num_cpus;

lazy_static! {
    /// XXX: We currently cannot have more than one event loop because of the shared socket
    /// issue.
    pub static ref EVENT_LOOP: Mutex<EventLoop<Reactor>> = {
        Mutex::new(EventLoop::new().unwrap())
    };

    pub static ref SENDER: Sender<Message> = {
        EVENT_LOOP.lock().unwrap().channel()
    };
}
