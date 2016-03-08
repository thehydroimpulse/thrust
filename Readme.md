# Thrust [![Build Status](https://travis-ci.org/thehydroimpulse/thrust.svg?branch=master)](https://travis-ci.org/thehydroimpulse/thrust)

**Note:** A work in progress. It's not in a useful state right now.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages.

## Features

- Scalable Thrift RPC
- Rust code generator from a `.thrift` file.
- Built on-top of Asynchronous I/O via Mio.
- Heavily uses Futures to manage asynchronous code.
- Multiplexing multiple RPC services on a single event-loop.

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

## License

MIT &mdash; go ham!
