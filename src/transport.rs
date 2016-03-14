use result::{ThrustResult};

pub trait Transport {
    fn open(&mut self) -> ThrustResult<()>;
    fn close(&mut self) -> ThrustResult<()>;
    fn read(&mut self, buf: &mut [u8]) -> ThrustResult<usize>;
    fn write(&mut self, buf: &[u8]) -> ThrustResult<()>;
    fn flush(&mut self) -> ThrustResult<()>;
}
