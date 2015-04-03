use nom::{Consumer, ConsumerState, MemProducer, IResult, Needed, space, multispace, alphanumeric, digit, line_ending};
use nom::IResult::*;
use std::str;
use std::str::{from_utf8};

named!(namespace_parser<&[u8], &str>,
  chain!(
    tag!("namespace") ~
    space ~
    name: map_res!(alphanumeric, from_utf8) ~
    tag!(";") ~
    multispace?,
    || { name }
  )
);

named!(identifier_parser<&[u8], &str>,
  chain!(
    iden: map_res!(alt!(
      tag!("struct") |
      tag!("service") |
      tag!("include") |
      tag!("typedef") |
      tag!("const") |
      tag!("enum") |
      tag!("exception")
    ), from_utf8),
    || { iden }
  )
);

named!(struct_parser<&[u8], StructIdent>,
  chain!(
    tag!("struct") ~
    space ~
    name: map_res!(alphanumeric, from_utf8) ~
    multispace? ~
    tag!("{") ~
    multispace? ~
    fields: many0!(struct_field_parser) ~
    multispace? ~
    tag!("}"),
    || {
        StructIdent {
            name: name.to_string(),
            fields: fields
        }
    }
  )
);

#[derive(PartialEq, Debug)]
pub struct StructIdent {
    name: String,
    fields: Vec<Field>
}

#[derive(PartialEq, Debug)]
pub struct Field {
    order: u8,
    optional: bool,
    ty: String,
    name: String
}

named!(struct_field_parser<&[u8], Field>,
  chain!(
    multispace? ~
    order: map_res!(digit, from_utf8) ~
    tag!(":") ~
    space? ~
    ty: map_res!(alphanumeric, from_utf8) ~
    space ~
    field: map_res!(alphanumeric, from_utf8) ~
    space? ~
    tag!(","),
    || {
        Field {
            order: order.parse().unwrap(),
            optional: false,
            ty: ty.to_string(),
            name: field.to_string()
        }
    }
  )
);

#[derive(PartialEq, Eq, Debug)]
pub enum State {
    // We haven't parsed anything yet, so we're at the very
    // beginning state-wise.
    Begin,
    Forms,
    End,
    Done
}

pub struct ParserConsumer {
    state: State,
    namespace: Option<String>
}

impl Consumer for ParserConsumer {
    fn consume(&mut self, input: &[u8]) -> ConsumerState {
        match self.state {
            State::Begin => {
                match namespace_parser(input) {
                    Done(_, ns) => {
                        // We have parsed the namespace, so we're now in a Fresh state.
                        // XXX: Thrift might actually allow multiple namespaces in a file...
                        self.state = State::Forms;
                        self.namespace = Some(ns.to_string());

                        // Note that we parsed some input and fill the buffer with another 5
                        // to start off with.
                        ConsumerState::Await(input.len(), 1)
                    },
                    Incomplete(Needed::Size(size)) => {
                        let len = input.len() + size as usize;
                        ConsumerState::Await(0, len + 1)
                    },
                    _ => {
                        // It's ok, we don't need to find a namespace.
                        self.state = State::Forms;
                        ConsumerState::Await(0, 5)
                    }
                }
            },
            State::Forms => {
                println!("state forms {:?}", from_utf8(input));
                match identifier_parser(input) {
                    Done(_, "struct") => {
                        println!("found struct");
                        ConsumerState::Await(input.len(), 1)
                    },
                    Done(_, b) => {
                        println!("got form");
                        ConsumerState::ConsumerDone
                    },
                    Incomplete(Needed::Size(size)) => {
                        let len = input.len() + size as usize;
                        ConsumerState::Await(0, len)
                    },
                    _ => {
                        self.state = State::Done;
                        ConsumerState::ConsumerDone
                    }
                }
            },
            _ => {
                ConsumerState::ConsumerDone
            }
        }
    }

    fn end(&mut self) {
        self.state = State::Done;
        println!("parser ended");
    }
}


#[test]
fn parse_namespace() {
    let mut p = MemProducer::new(b"namespace foogggggbar;", 5);
    let mut c = ParserConsumer { state: State::Begin, namespace: None };
    c.run(&mut p);

    assert_eq!(c.namespace, Some("foogggggbar".to_string()));
    assert_eq!(c.state, State::Done);
}

#[test]
fn parse_struct() {
    let input = &b"struct Foobar {\n1: i32 foobar,\n }"[..];
    assert_eq!(struct_parser(input), IResult::Done(
        &b""[..],
        StructIdent {
            name: "Foobar".to_string(),
            fields: vec![
                Field {
                    order: 1,
                    optional: false,
                    ty: "i32".to_string(),
                    name: "foobar".to_string()
                }
            ]
        }
    ));
}

#[test]
fn parse_struct_two_fields() {
    let input = &b"struct Foobar {\n1: i32 foobar,\n2: i64 bigbo,\n }"[..];
    assert_eq!(struct_parser(input), IResult::Done(
        &b""[..],
        StructIdent {
            name: "Foobar".to_string(),
            fields: vec![
                Field {
                    order: 1,
                    optional: false,
                    ty: "i32".to_string(),
                    name: "foobar".to_string()
                },
                Field {
                    order: 2,
                    optional: false,
                    ty: "i64".to_string(),
                    name: "bigbo".to_string()
                }
            ]
        }
    ));
}

#[test]
fn parse_struct_field() {
    let input = &b"1: i32 foobar,"[..];
    assert_eq!(struct_field_parser(input), IResult::Done(
        &b""[..],
        Field {
            order: 1,
            optional: false,
            ty: "i32".to_string(),
            name: "foobar".to_string()
        }
    ));
}

#[test]
fn parse_idents() {
    let idents = vec![
        &b"struct"[..],
        &b"service"[..],
        &b"include"[..],
        &b"typedef"[..],
        &b"exception"[..],
        &b"enum"[..]
    ];

    for iden in idents {
        assert_eq!(identifier_parser(iden), IResult::Done(
            &b""[..],
            from_utf8(iden).unwrap()
        ));
    }
}

