use protocol::{Deserializer, ThriftDeserializer, ThriftMessage, Error};
use tangle::Future;

pub trait Runner {
    fn run<D>(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error>
        where D: Deserializer + ThriftDeserializer;
}
