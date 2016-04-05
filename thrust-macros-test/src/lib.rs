#![feature(plugin)]
#![plugin(thrust_macros)]

extern crate thrust;

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

    service FlockDb {
        map<string, byte> query(1: string voodoo);
    }
");

#[test]
fn compile() {
    let m = foobar1::Foo::HELLO;
    println!("{:?}", m);
}
