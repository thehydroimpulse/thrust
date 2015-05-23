pub trait Service {}

pub struct Server<'a, T> {
    service: T,
    addr: &'a str
}

impl<'a, T: Service> Server<'a, T> {
    pub fn new(addr: &'a str, service: T) -> Server<'a, T> {
        Server {
            service: service,
            addr: addr
        }
    }

    /// Bind the Thrift service to a given transport.
    pub fn listen(&self) {
    }
}
