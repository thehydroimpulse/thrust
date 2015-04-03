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
thrust!(<namespace>, <str>);
```

Where `<namespace>` is the Rust module that the generated code will be located at and `<str>` is where your Thrift definitions would go.

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

Services are implemented with the use of traits. For each service, a trait of the same name will be generated under the specified Thrust namespace.

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
