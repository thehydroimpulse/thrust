# Thrust

**Note:** A work in progress. It's not in a useful state right now.

A Rust implementation of the [Apache Thrift](https://thrift.apache.org/) protocol that simplifies communicating between independent systems that may be implemented in various languages.

## Features

- Scalable Thrift RPC
- Rust code generator from a `.thrift` file.
- Built on-top of Asynchronous I/O via Mio.
- Heavily uses Futures to manage asynchronous code.

## Installing

```toml
[dependencies]
thrust = "*"
```

## License

MIT &mdash; go ham!
