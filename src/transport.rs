use std::io::{Read, Write};
use server::Task;
use std::sync::mpsc::{Sender};

pub trait Stream: Write + Read {}

pub trait Transport {
    type Connection: Stream;
    fn listen(&mut self, tx: Sender<Task<Self::Connection>>);
}
