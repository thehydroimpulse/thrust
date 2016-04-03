#![feature(plugin)]
#![plugin(thrust_macros)]

thrust!("
    namespace rust foobar1

    struct Flocker {
        1: required i64 fo;
    }

    enum Foo {
        HELLO,
        Foobar
    }

    struct Message {
        1: required i32 foobar;
    }
");

#[test]
fn compile() {
    let m = foobar1::Foo::HELLO;
    println!("{:?}", m);
}
