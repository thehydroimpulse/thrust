# Thrust

**Note:** A work in progress. The current focus is on the parser for the IDL before moving onto the macros and such.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages. Now, you can easily integrate Rust into your toolbox if you're heavily using Thrift.

**Warning:** Because Thrust uses syntax extensions to generate code during the compile phase, you'll need to stick to the Rust nightlies for Thrust to compile. We'll be working on a separate code generation method that doesn't require syntax extensions for Thrust to be usable on 1.0.

## Installing

```toml
[dependencies]
thrust = "0.1.0"
```

## Getting Started

Thrust takes advantage of Rust's macros and doesn't touch the file system while generating the required code. It just works!

The single macro that Thrust uses has the following definition:

```rust
thrust!("<str>");
```

You would replace `<str>` with your Thrift definitions.

## Definitions

You would place your definitions normally, like you would in any other language. If you'd rather have your thrift files separate from the specific language source, that's totally ok! All Thrust requires is that you provide a string of some sort to the macro. This allows one to use the `include_str!` built-in Rust macro to suite your needs.

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

Now let's define the Rust file that can be located anywhere. Again, Thrust doesn't generate any extra files, so it's completely agnostic to how your file system is structured.

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

Thrust currently only supports raw TCP. HTTP can be added later using the Hyper HTTP library.

Asynchronous, event-driven I/O is not currently implemented, but will certainly be an option in the future.

## Protocol

Thrust supports JSON and the simple binary encodings that are common within Thrift. Support for compact binary or other options will be supported in the future.

## Services

Services are implemented with the use of traits. For each service, a trait of the same name will be generated under the specified Thrust namespace.

```rust
#[macro_use]
extern crate thrust;

use thrust::Service;

thrust!("
  namespace rust wonder;

  struct Person {
    1: string name,
  }

  service Auth {
    bool isAuthenticated(),
    string login(1:string email, 2:string password)
  }
");

struct Auth;

impl wonder::Auth for Auth {
  fn isAuthenticated() -> bool {
    // ...
  }

  fn login(email: String, password: String) -> String {
    // ...
  }
}
```

## Creating a Server

Thrust servers accept a transport and a protocol. The server's job is to initialize the different components, passing the grunt of the work to both the transport and the protocol.

```rust
#[macro_use]
extern create thrust;

use thrust::{Server, TcpTransport, Protocol};

fn main() {
  let mut transport = TcpTransport::new("0.0.0.0", 5688");
  let mut server = Server::new(&mut transport, Protocol::Binary);

  // Start listening for connections and hand them off to a Processor.
  server.listen();
}
```

## Concepts

When a transport receives a connection, it will pass it off to the Processor in a separate thread. Each transport exposes a set of uniform methods for dealing with a common set of functions. That allows other units in the system to be unaware of the type of transport and type of protocol.

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
