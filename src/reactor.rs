use mio::Handler;

pub struct Reactor;

impl Handler for Reactor {
    type Timeout = ();
    type Message = ();
}
