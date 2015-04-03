# Thrust

**Note:** A work in progress. The current focus is on the parser for the IDL before moving onto the macros and such.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that handles Thrift's IDL and generates the RPC system to communicate with other Thrift services.

## Installing

```toml
[dependencies]
thrust = "0.1.0"
```

## Getting Started

Thrust takes advantage of Rust's macros and doesn't touch the file system while generating the RPC code. This allows you to enjoy a world without any generated files to worry about, build commands to issue, etc...

```rust
#[macro_use]
extern crate thrust;

thrust!(Thrift, "
  struct Person {
    1: string name,
  }
");

fn main() {
  let p = Thrift::Person {
    name: "foobar".to_string()
  }
}
```

That's it. When you compile your program, the macro will take care of parsing the definitions and generate the appropriate code.


## Services

Services are implemented through Traits in Rust. You'll simply implement the appropriate service trait and you'll be good to go.

```rust
#[macro_use]
extern crate thrust;

use thrust::Service;

thrust!(Thrift, "
  struct Person {
    1: string name,
  }

  service Auth {
    bool isAuthenticated(),
    string login(1:string email, 2:string password)
  }
");

struct Auth;

impl Thrift::Auth for Auth {
  fn isAuthenticated() -> bool {
    // ...
  }

  fn login(email: String, password: String) -> String {
    // ...
  }
}
```

## License

MIT, go ham!
