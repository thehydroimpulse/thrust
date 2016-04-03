#![feature(plugin)]
#![plugin(thrust_macros)]

thrust!("
    namespace rust foobar1

    enum Foo {
        HELLO,
        Foobar
    }

    struct Message {
        1: required binary foobar;
        2: optional string big;
    }
");

#[test]
fn compile() {
    let m = foobar1::Foo::HELLO;
}
