# Thrust [![Build Status](https://travis-ci.org/thehydroimpulse/thrust.svg?branch=master)](https://travis-ci.org/thehydroimpulse/thrust)

**Note:** A work in progress. It's not in a useful state right now.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages.

## Features

- Scalable Thrift RPC
- Rust code generator from a `.thrift` file.
- Built on-top of Asynchronous I/O via Mio.
- Heavily uses Futures to manage asynchronous code.
- Multiplexing multiple RPC services on a single event-loop.
- (Currently limited to one for now) Automatically spawn an `EventLoop` per CPU core.

## Installing

```toml
[dependencies]
thrust = "*"
```

You may also want to install the code generator using Cargo:

```bash
cargo install thrust
```

## Generating Rust Code

Thrust comes with a `thrust` binary that compiles your `.thrift` files into Rust code.

```bash
thrust hello.thrift .
```

The first argument is the input thrift file and the second is the *path* where you want your
Rust file to be written to. The filename will be based on the Rust namespace in your thrift file `namespace rust <name>`.

## Spawning The Reactor

All the I/O and networking is built using Mio. By default, a single Reactor is spun up globally. Support for multiple event loops/Reactors are supported but not working correctly right now.

```rust
use thrust::Reactor;

fn main() {
  // Run the reactor that multiplexes many clients and servers.
  Reactor::run();
}
```

## Creating a Thrift Service

Thrust supports creating Thrift services, backed by non-blocking TCP sockets with Mio.

```thrift
namespace rust thrift;
// Start by defining a service in your Thrift file.
service Flock {
  bool isLoggedIn(1: string token);
}
```

After using Thrust to generate the service in Rust, we can start using it.

```rust
extern crate thrust;
// Tangle is a futures implementation
extern crate tangle;

// The generated Rust module.
use thrift::Flock;

use thrust::{Reactor, Server};
use tangle::{Future, Async};

pub struct FlockService;

impl Flock for FlockService {
  fn isLoggedIn(&mut self, token: String) -> Future<bool> {
    if &*token == "123" {
      Async::Ok(true)
    } else {
      Async::Ok(false)
    }
  }
}

fn main() {
  let addr: SocketAddr = "127.0.0.1:7899".parse().unwrap();

  // Asynchronously bind the server to the specified port. This does
  // not block the current thread.
  Server::new(FlockService).bind(addr);

  // Run the Reactor with the server.
  Reactor::run();
}
```

## License

MIT &mdash; go ham!
