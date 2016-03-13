use protocol::{Deserializer, ThriftDeserializer, ThriftMessage, Error};
use tangle::Future;

pub trait Caller<D>
    where D: Deserializer + ThriftDeserializer
{
    fn call(&mut self, de: &mut D, msg: ThriftMessage) -> Result<Future<Vec<u8>>, Error>;
}
