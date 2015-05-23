# Thrust

**Note:** A work in progress. It's not in a useful state right now.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages. Now, you can easily integrate Rust into your toolbox if you're heavily using Thrift.

**Warning:** Because Thrust uses syntax extensions to generate code during the compile phase, you'll need to stick to the Rust nightlies for Thrust to compile. A traditional file generator is an option for the future to support stable 1.0.

## Installing

```toml
[dependencies]
thrust = "*"
```

## Getting Started

Thrust takes advantage of Rust's macros and doesn't touch the file system while generating the required code.

The single macro that thrust uses has the following definition:

```rust
thrust!("<str>");
```

You would replace `<str>` with your Thrift definitions.

## IDL

You would place your thrift IDL normally, like you would in any other language. If you'd rather have your thrift files separate from the specific language source, that's totally ok! All thrust requires is that you provide a string of some sort to the macro. This allows one to use the `include_str!` built-in Rust macro to suite your needs.

Let's define a simple Thrift file that can be used across both a C++ project and a Rust project:

```thrift
// thrift/account.thrift
namespace rust Wonder;
namespace cpp wonder;

struct Account {
  1: string username,
  2: string password,
  3: string token,
}
```

Now let's define the Rust file that can be located anywhere. Again, thrust doesn't generate any extra files, so it's completely agnostic to how your file system is structured.

```rust
// src/lib.rs
#[macro_use]
extern crate thrust;

thrust!(include_str!("./../thrift/account.thrift"));

fn main() {
  let new_account = Wonder::Account {
    username: "foobar".to_string(),
    password: "21221".to_string(),
    token: "fj2jX2kSWGjaI".to_string()
  };
}
```

That's it. When you compile your program, the macro will take care of parsing the definitions and generate the appropriate code. Thrust will use either the first catch-all namespace `namespace foo;` or the first Rust targeted namespace to dump the generated code within. The previous example used the `Wonder` rust namespace, so that's how we'll access the generated items.

## Transports

Thrust currently only supports raw TCP.

Asynchronous, event-driven I/O is not currently implemented, but will certainly be an option in the future. A traditional blocking I/O with a thread-pool is the basis for the current implementation.

## Protocol

The simple thrift binary protocol is the only one supported. New protocols are fairly easy to add.

## Creating a Service

Each service spins up it's own server, isolated from any other service. Thrust uses the namespace declaration within the IDL for it's module name of the generated code.
Here, we're creating a new `flockdb.thrift` namespace specifically for the Rust language.

For each service, we'll have a trait and a struct generated. The trait ensures we're implementing all the RPC methods needed, and the struct gives us a target to implement the trait on.

```rust
/// An example that somewhat translates Twitter's use of Thrift within the
/// FlockDb database to Thrust, a Rust implementation of Thrift.
/// This should illustrate the rough API that Thrust exposes and how code generation
/// using the `thrust!` procedural macro works.

extern crate thrust;

use thrust::{Server, ThriftResult};
use flockdb::thrift::{FlockDb};

thrust!("
    namespace rust flockdb.thrift;

    service FlockDB {
        bool contains(1: i64 source_id, 2: i32 graph_id, 3: i64 destination_id);
    }
");

impl FlockDb::Service for FlockDb::Server {
    fn contains(source_id: i64, graph_id: i32, destination_id: i64) -> ThriftResult<bool> {
        Ok(true)
    }
}

fn main() {
    let server = Server::new("localhost:8000", FlockDb::Server);
    server.listen();
}
```

As you can see, creating services is really easy. The only implementation you need to write yourself is the IDL and the implementation.

## Re-exporting Thrift Types

By default, the generated "namespace" (in Thrift) or module (in Rust) is private. That means only the current file would be able to access the generated types. If you have multiple thrift namespaces, multiple thrift files, you might not want this behaviour. That's totally ok!

The recommended way is to call the `thrust!` macro is a separate file and re-export the private module.

```rust
// src/lib.rs

// Auth is where the authentication thrift types would be.
pub mod auth;
```

Finally, you're `lib/auth.rs`

```rust
// src/auth.rs

// You can either embed the thrift file here, or you can use the include_str! macro.
thrust!(...);

// Re-export the generated namespace.
pub use wonder;
```

Now you're free to use the types in any other file in your Rust project(s). If you heavily use Thrift, you can create a separate cargo project just for your thrift definitions.

## License

MIT &mdash; go ham!
