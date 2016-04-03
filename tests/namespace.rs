// extern crate thrust;

// use thrust::ThriftCompiler;

// pub const ns: &'static [u8] = b"namespace rust foobar\n";
// pub const ns_1: &'static [u8] = b"
// namespace rust foobar\n
// ";
// pub const ns_2: &'static [u8] = b"
//     namespace rust foobar\n
// ";
// pub const empty: &'static [u8] = b"";

// #[test]
// #[should_panic]
// fn ns_empty() {
//     let r = ThriftCompiler::run(empty).unwrap();
// }

// #[test]
// fn namespace() {
//     let r = ThriftCompiler::run(ns).unwrap();
//     assert_eq!(&*r.namespace, "foobar");
//     assert_eq!(&*r.buffer, "");
// }

// #[test]
// fn namespace_1() {
//     let r = ThriftCompiler::run(ns_1).unwrap();
//     assert_eq!(&*r.namespace, "foobar");
//     assert_eq!(&*r.buffer, "");
// }

// #[test]
// fn namespace_2() {
//     let r = ThriftCompiler::run(ns_2).unwrap();
//     assert_eq!(&*r.namespace, "foobar");
//     assert_eq!(&*r.buffer, "");
// }
