use rand::{OsRng, Rng};

pub struct Request {
    pub id: u64,
    buf: Vec<u8>
}

impl Request {
    pub fn new(buf: Vec<u8>) -> Request {
        Request {
            id: OsRng::new().unwrap().next_u64(),
            buf: buf
        }
    }
}
