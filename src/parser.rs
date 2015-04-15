use nom::{Consumer, ConsumerState, MemProducer, IResult, Needed, space, multispace, alphanumeric, digit, line_ending};
use nom::IResult::*;
use std::str;
use std::str::{from_utf8};

#[derive(PartialEq, Debug)]
pub struct Field {
    order: u8,
    optional: bool,
    ty: String,
    name: String
}

#[derive(PartialEq, Eq, Debug)]
pub enum State {
    // We haven't parsed anything yet, so we're at the very
    // beginning state-wise.
    Begin,
    Forms,
    End,
    Done
}

#[derive(Debug, PartialEq)]
pub enum Ast {
    /// (Lang, Namespace)
    Namespace(Vec<String>),
    Struct(String, Vec<Field>),
    Typedef(ThriftType, String)
}

pub struct ParserConsumer {
    state: State,
    namespace: Option<String>
}

#[derive(Debug, PartialEq)]
pub enum ThriftType {
    Bool,
    Byte,
    S16Int,
    S32Int,
    S64Int,
    Double,
    String,
    Binary,
    Map,
    List,
    Set
}

impl ThriftType {
    pub fn from_string(input: String) -> ThriftType {
        match &input[..] {
            "i16" => ThriftType::S16Int,
            "i32" => ThriftType::S32Int,
            "i64" => ThriftType::S64Int,
            "double" => ThriftType::Double,
            "string" => ThriftType::String,
            "binary" => ThriftType::Binary,
            "bool" => ThriftType::Bool,
            "byte" => ThriftType::Byte,
            _ => panic!("Thrust: the type '{}' is not a real type.", input)
        }
    }
}

named!(pub namespace_parser<&[u8], Ast>,
  chain!(
    tag!("namespace") ~
    parts: many1!(
        chain!(
            space ~
            name: map_res!(alphanumeric, from_utf8),
            || { name.to_string() }
        )
    ) ~
    line_ending,
    || {
        Ast::Namespace(parts)
    }
  )
);

// XXX: Implement the more complex types like
//      map, list and set. These will have their
//      own named functions to parse `ty<0, ...n>`
named!(pub types<&[u8], String>,
  chain!(
    val: map_res!(alt!(
      tag!("i16") |
      tag!("i32") |
      tag!("i64") |
      tag!("bool") |
      tag!("byte") |
      tag!("double") |
      tag!("string") |
      tag!("binary")
    ), from_utf8),
    || { val.to_string() }
  )
);

named!(pub typedef_parser<&[u8], Ast>,
  chain!(
    tag!("typedef") ~
    multispace ~
    ty: types ~
    multispace ~
    alias: map_res!(alphanumeric, from_utf8) ~
    line_ending,
    || {
        Ast::Typedef(ThriftType::from_string(ty), alias.to_string())
    }
  )
);

named!(pub identifier_parser<&[u8], &str>,
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

named!(pub struct_parser<&[u8], Ast>,
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
        Ast::Struct(name.to_string(), fields)
    }
  )
);

named!(pub struct_field_parser<&[u8], Field>,
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


impl Consumer for ParserConsumer {
    fn consume(&mut self, input: &[u8]) -> ConsumerState {
        match self.state {
            // State::Begin => {
            //     match namespace_parser(input) {
            //         Done(_, ns) => {
            //             // We have parsed the namespace, so we're now in a Fresh state.
            //             // XXX: Thrift might actually allow multiple namespaces in a file...
            //             self.state = State::Forms;
            //             self.namespace = Some(ns.to_string());

            //             // Note that we parsed some input and fill the buffer with another 5
            //             // to start off with.
            //             ConsumerState::Await(input.len(), 1)
            //         },
            //         Incomplete(Needed::Size(size)) => {
            //             let len = input.len() + size as usize;
            //             ConsumerState::Await(0, len + 1)
            //         },
            //         _ => {
            //             // It's ok, we don't need to find a namespace.
            //             self.state = State::Forms;
            //             ConsumerState::Await(0, 5)
            //         }
            //     }
            // },
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


#[cfg(test)]
mod test {
    use super::*;

    use nom::{Consumer, ConsumerState, MemProducer, IResult, Needed, space, multispace, alphanumeric, digit, line_ending};
    use nom::IResult::*;

    #[test]
    fn parse_namespace() {
        let input = &b"namespace rust foobar\n"[..];
        assert_eq!(namespace_parser(input), IResult::Done(
            &b""[..],
            Ast::Namespace(vec!["rust".to_string(), "foobar".to_string()])
        ));
    }

    #[test]
    fn parse_typedefs() {
        let mut input = &b"typedef i32 MyInteger\n"[..];
        assert_eq!(typedef_parser(input), IResult::Done(
            &b""[..],
            Ast::Typedef(ThriftType::S32Int, "MyInteger".to_string())
        ));
    }

    #[test]
    fn parse_struct() {
        let input = &b"struct Foobar {\n1: i32 foobar,\n }"[..];
        let fields = vec![
            Field {
                order: 1,
                optional: false,
                ty: "i32".to_string(),
                name: "foobar".to_string()
            }
        ];

        assert_eq!(struct_parser(input), IResult::Done(
            &b""[..],
            Ast::Struct("Foobar".to_string(), fields)
        ));
    }

    #[test]
    fn parse_struct_two_fields() {
        let input = &b"struct Foobar {\n1: i32 foobar,\n2: i64 bigbo,\n }"[..];
        let fields = vec![
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
        ];

        assert_eq!(struct_parser(input), IResult::Done(
            &b""[..],
            Ast::Struct("Foobar".to_string(), fields)
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
}
