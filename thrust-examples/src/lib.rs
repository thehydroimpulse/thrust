extern crate thrust;
extern crate tangle;

pub mod foobar1;
use foobar1::{
    BlizzardService,
    BlizzardClient,
    BlizzardServer
};

use thrust::Reactor;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use tangle::{Async, Future};

struct Blizzard;

impl BlizzardService for Blizzard {
    fn ack(&mut self, source_id: i64, tuple_id: i64) -> Future<String> {
        println!("Received an ack from source[{}] and tuple[{}]", source_id, tuple_id);
        Future::unit("ack was successful!".to_string())
    }
}

#[test]
fn create_a_client() {
    let addr: SocketAddr = "127.0.0.1:2767".parse().unwrap();
    let server = BlizzardServer::new(Blizzard, addr.clone());

    thread::sleep(Duration::from_millis(25));
    let mut rpc = BlizzardClient::new(addr.clone());

    rpc.ack(45, 99).and_then(move |res| {
        println!("{:?}", res);
        Async::Ok(())
    });

    Reactor::run().join();
    rpc.handle.join();
    server.handle.join();
}
