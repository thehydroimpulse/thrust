use protocol::{Deserializer, ThriftDeserializer, ThriftMessage, Error};
use tangle::Future;

pub trait Runner<D>
    where D: Deserializer + ThriftDeserializer
{
    fn run(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error>;
}
