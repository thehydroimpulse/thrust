# Thrust [![Build Status](https://travis-ci.org/thehydroimpulse/thrust.svg?branch=master)](https://travis-ci.org/thehydroimpulse/thrust)

**Note:** A work in progress. It's not in a useful state right now.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages.

## Features

- Scalable Thrift RPC
- Rust code generator from a `.thrift` file.
- Built on-top of Asynchronous I/O via Mio.
- Heavily uses Futures to manage asynchronous code.
- Multiplexing multiple RPC services on a single event-loop.
- Automatically spawn an `EventLoop` per CPU core.

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

## Spawning Event Loops

```rust
use thrust::Spawner;

fn main() {
  // By default, this will run as many event loop threads as you have CPU cores.
  let spawner = Spawner::new(None);

  // ...

  // Block the main thread until all event loops terminate.
  spawner.join();
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

use thrust::{Spawner, Server};
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
  // Create a Spawner to manage Mio event loops.
  let mut spawner = Spawner::new(None);

  Server::run(&spawner, FlockService);

  spawner.join();
}
```

## License

MIT &mdash; go ham!

