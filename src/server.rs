use runner::Runner;
use protocol::{Deserializer, ThriftDeserializer};
use binary_protocol::BinaryDeserializer;
use std::io::Cursor;

pub type Default = BinaryDeserializer<Cursor<Vec<u8>>>;

/// Manages incoming RPC requests from a Mio event loop and dispatches it
/// to a runner that will then deserialize the Thrift message and call the appropriate
/// RPC function.
///
/// The server will also manage the response coming back from the RPC method through
/// the use of futures. These will be coordinated back to Mio.
pub struct Server<R: Runner<Default>> {
    de: Default,
    runner: R
}

impl<R> Server<R>
    where R: Runner<Default>
{
    pub fn new(runner: R) -> Server<R> {
        Server {
            de: BinaryDeserializer::new(Cursor::new(Vec::new())),
            runner: runner
        }
    }
}
