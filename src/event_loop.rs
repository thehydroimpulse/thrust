use mio::EventLoop;
use reactor::Reactor;

lazy_static! {
    /// A single event loop is used for multiple RPC services, effectively
    /// multiplexing connections onto the single event loop. This also allows
    /// one to use a single thread for all networking related tasks and worry
    /// less about multi-threading.
    static ref EVENT_LOOP: EventLoop<Reactor> = {
        EventLoop::new().unwrap()
    };
}
