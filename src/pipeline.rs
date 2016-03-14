use protocol::{Error, ThriftDeserializer, Deserializer, ThriftMessageType, ThriftMessage};
use tangle::{Future, Async};
use runner::Runner;

pub struct MessagePipeline<D> {
    de: D
}

impl<D> MessagePipeline<D>
    where D: Deserializer + ThriftDeserializer
{
    pub fn new(de: D) -> MessagePipeline<D> {
        MessagePipeline {
            de: de
        }
    }

    /// XXX: The fn signature should be `Result<Future<Vec<u8>>, Error>` where the serialized
    /// response is returned into the future.
    pub fn run(&mut self, runner: &mut Runner<D>) -> Result<Future<Vec<u8>>, Error> {
        let msg = try!(self.de.read_message_begin());

        match msg.ty {
            // Dispatch on an RPC method call.
            ThriftMessageType::Call => {
                runner.run(&mut self.de, msg)
            },
            _ => {
                panic!("unexpected");
            }
        }
    }
}
