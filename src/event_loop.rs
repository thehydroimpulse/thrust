use mio::{EventLoop, Sender};
use reactor::{Reactor, Message};
use std::sync::Arc;


lazy_static! {
    /// XXX: There should be an option to spin up a separate event loop for
    /// each CPU core. That means we would spin up `System` that would spin
    /// up the event loops and store their respective `Sender`s.

    /// A single event loop is used for multiple RPC services, effectively
    /// multiplexing connections onto the single event loop. This also allows
    /// one to use a single thread for all networking related tasks and worry
    /// less about multi-threading.
    pub static ref EVENT_LOOP: EventLoop<Reactor> = {
        EventLoop::new().unwrap()
    };
    pub static ref SENDER: Sender<Message> = EVENT_LOOP.channel();
}
