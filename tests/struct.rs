extern crate thrust;

use thrust::ThriftCompiler;

pub const empty: &'static [u8] = b"
namespace rust ping
struct Ping {}
";

pub const empty_gen: &'static str = "pub struct Ping {
}
";

pub const field: &'static [u8] = b"
namespace rust ping
struct Ping {
    1: required string foobar;
}
";

pub const field_gen: &'static str = "pub struct Ping {
    foobar: String,
}
";

#[test]
fn test_empty_struct() {
    let r = ThriftCompiler::run(empty).unwrap();
    assert_eq!(&*r.namespace, "ping");
    assert_eq!(&*r.buffer, empty_gen);
}

#[test]
fn test_field_struct() {
    let r = ThriftCompiler::run(field).unwrap();
    assert_eq!(&*r.namespace, "ping");
    assert_eq!(&*r.buffer, field_gen);
}
