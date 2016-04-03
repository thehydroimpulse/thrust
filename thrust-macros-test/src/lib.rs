#![feature(plugin)]
#![plugin(thrust_macros)]

thrust!("
    namespace rust foobar1

    enum Foo {
        HELLO,
        Foobar
    }
");

#[test]
fn compile() {
    let m = foobar1::Foo::HELLO;
}
