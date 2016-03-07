use nom::{space, alphanumeric, ErrorKind, IterIndices, IResult, multispace, not_line_ending, Err, be_u8, digit, InputLength, is_alphanumeric};
use nom::IResult::*;
use std::str;
use std::convert::From;
use std::ops::{Index,Range,RangeFrom};

use ast::{IdentNode, StructNode, FunctionNode, ServiceNode, NamespaceNode, StructFieldNode, FieldMetadataNode, Ty, Ast};

named!(blanks,
       chain!(
           many0!(alt!(multispace | comment_one_line | comment_block)),
           || { &b""[..] }));

// Auxiliary parser to ignore one-line comments
named!(comment_one_line,
       chain!(
           alt!(tag!("//") | tag!("#")) ~
           not_line_ending? ~
           alt!(eol | eof),
           || { &b""[..] }));

named!(eol,
       alt!(tag!("\r\n") | tag!("\n") | tag!("\u{2028}") | tag!("\u{2029}")));

// Auxiliary parser to ignore block comments
named!(comment_block,
       chain!(
           tag!("/*") ~
           take_until_and_consume!(&b"*/"[..]),
           || { &b""[..] }));

fn eof(input:&[u8]) -> IResult<&[u8], &[u8]> {
    if input.len() == 0 {
        Done(input, input)
    } else {
        Error(Err::Code(ErrorKind::Eof))
    }
}

named!(pub parse_ident<&[u8], &str>, map_res!(alphanumeric, str::from_utf8));

/// Recognizes numerical and alphabetic characters: 0-9a-zA-Z[.]
pub fn namespace<'a>(input:&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
  let input_length = input.input_len();
  if input_length == 0 {
    return Error(Err::Position(ErrorKind::AlphaNumeric, input));
  }

  for (idx, item) in input.iter_indices() {
    if !is_alphanumeric(*item) || *item != b'.' {
      if idx == 0 {
        return Error(Err::Position(ErrorKind::AlphaNumeric, input))
      } else {
        return Done(&input[idx..], &input[0..idx])
      }
    }
  }

  Done(&input[input_length..], input)
}

named!(pub parse_namespace<&[u8], NamespaceNode>, chain!(
    tag!("namespace") ~
    space ~
    lang: parse_ident ~
    space ~
    ns: map_res!(namespace, str::from_utf8),
    || {
        NamespaceNode::new(IdentNode(lang.to_string()), ns.to_string())
    }
));

pub type Generics = Vec<Ty>;

named!(pub parse_generic_types<&[u8], Vec<String> >, chain!(
    ty: separated_list!(tag!(","), chain!(
        multispace? ~
        ty: map_res!(alphanumeric, str::from_utf8),
        || { ty }
    )),
    || { ty.into_iter().map(|s| s.to_string()).collect() }
));

named!(pub parse_generics<&[u8], Generics>, chain!(
    types: delimited!(tag!("<"), parse_generic_types, tag!(">")),
    || { types.into_iter().map(|ty| Ty::parse(ty, vec![])).collect() }
));

named!(pub parse_simple_type<&[u8], String>, chain!(
    ty: map_res!(alphanumeric, str::from_utf8),
    || { ty.to_string() }
));

named!(pub parse_type<&[u8], Ty>, chain!(
    ty: parse_simple_type ~
    generics: parse_generics?,
    || { Ty::parse(ty, generics.unwrap_or(Vec::new())) }
));

named!(pub parse_metadata<&[u8], FieldMetadataNode>, chain!(
    meta: map_res!(alt!(tag!("optional") | tag!("required")), str::from_utf8) ~
    space,
    || { FieldMetadataNode::parse(meta) }
));

named!(pub parse_service_method<&[u8], FunctionNode>, chain!(
    multispace? ~
    ty: parse_type ~
    multispace ~
    name: parse_ident ~
    multispace? ~
    tag!("(") ~
    multispace? ~
    tag!(")") ~
    multispace? ~
    alt!(tag!(",") | tag!(";")),

    || {
        FunctionNode {
            name: IdentNode(name.to_string()),
            ret: ty
        }
    }
));

named!(pub parse_service<&[u8], ServiceNode>, chain!(
    tag!("service") ~
    multispace ~
    name: parse_ident ~
    multispace? ~
    tag!("{") ~
    multispace? ~
    methods: many0!(parse_service_method) ~
    multispace? ~
    tag!("}"),
    || {
        ServiceNode {
            name: IdentNode(name.to_string()),
            methods: methods
        }
    }
));

/// ```notrust
/// struct Foobar {
///     1: string name
/// }
/// ```
named!(pub parse_struct<&[u8], StructNode>, chain!(
    tag!("struct") ~
    space ~
    name: parse_ident ~
    space ? ~
    tag!("{") ~
    multispace? ~
    fields: many0!(parse_struct_field) ~
    multispace? ~
    tag!("}"),
    || {
        StructNode {
            name: IdentNode(name.to_string()),
            fields: fields
        }
    }
));

named!(pub parse_struct_field<&[u8], StructFieldNode>, chain!(
    order: map_res!(
        map_res!(
            digit,
            str::from_utf8
        ),
        str::FromStr::from_str
    ) ~
    tag!(":") ~
    space ~
    metadata: parse_metadata ~
    ty: parse_type ~
    space? ~
    ident: parse_ident ~
    tag!(";") ~
    multispace?,
    || {
        StructFieldNode {
            order: order,
            metadata: metadata,
            ty: ty,
            ident: IdentNode(ident.to_string())
        }
    }
));

pub struct Parser<'a> {
    input: &'a str
}

mod tests {
    use super::*;
    use std::str;
    use nom::IResult::*;

    use ast::{IdentNode, StructNode, FunctionNode, ServiceNode, StructFieldNode, FieldMetadataNode, Ty, Ast};

    #[test]
    fn should_parse_generics() {
        assert_eq!(parse_generics(b"<void>"), Done(&[][..], vec![Ty::Void]));
        assert_eq!(parse_generics(b"<string>"), Done(&[][..], vec![Ty::String]));
        assert_eq!(parse_generics(b"<i32>"), Done(&[][..], vec![Ty::Signed32]));
    }

    #[test]
    fn parse_simple_types() {
        assert_eq!(parse_simple_type(b"void"), Done(&b""[..], format!("void")));
        assert_eq!(parse_simple_type(b"bool"), Done(&b""[..], format!("bool")));
        assert_eq!(parse_simple_type(b"i16"), Done(&b""[..], format!("i16")));
        assert_eq!(parse_simple_type(b"i32"), Done(&b""[..], format!("i32")));
        assert_eq!(parse_simple_type(b"i64"), Done(&b""[..], format!("i64")));
        assert_eq!(parse_simple_type(b"double"), Done(&b""[..], format!("double")));
        assert_eq!(parse_simple_type(b"binary"), Done(&b""[..], format!("binary")));
        assert_eq!(parse_simple_type(b"string"), Done(&b""[..], format!("string")));
    }

    #[test]
    fn parse_types() {
        assert_eq!(parse_type(b"void "), Done(&[32][..], Ty::Void));
        assert_eq!(parse_type(b"bool "), Done(&[32][..], Ty::Bool));
        assert_eq!(parse_type(b"i16 "), Done(&[32][..], Ty::Signed16));
        assert_eq!(parse_type(b"i32 "), Done(&[32][..], Ty::Signed32));
        assert_eq!(parse_type(b"i64 "), Done(&[32][..], Ty::Signed64));
        assert_eq!(parse_type(b"double "), Done(&[32][..], Ty::Double));
        assert_eq!(parse_type(b"binary "), Done(&[32][..], Ty::Binary));
        assert_eq!(parse_type(b"list<string>"), Done(&[][..], Ty::List(Box::new(Ty::String))));

        let map_ty1 = Box::new(Ty::Binary);
        let map_ty2 = Box::new(Ty::Signed16);
        assert_eq!(parse_type(b"map<binary,i16>"), Done(&[][..], Ty::Map(map_ty1, map_ty2)));
    }

    #[test]
    #[should_panic]
    fn should_fail_string_generic() {
        assert_eq!(parse_type(b"string<string> "), Done(&[62][..], Ty::String));
    }

    #[test]
    fn define_service() {
        let r = parse_service(b"service Foobar {}");
        let node = ServiceNode::new(IdentNode(format!("Foobar")));
        assert_eq!(r, Done(&b""[..], node));
    }

    #[test]
    fn define_service_ws() {
        let r = parse_service(b"service   Foobar  {\n}\n");
        let node = ServiceNode::new(IdentNode(format!("Foobar")));
        assert_eq!(r, Done(&[10][..], node));
    }

    #[test]
    fn define_service_method() {
        let r = parse_service(b"service Foobar {
            void ping();
        }");

        let method = FunctionNode { name: IdentNode(format!("ping")), ret: Ty::Void };
        let node = ServiceNode { name: IdentNode(format!("Foobar")), methods: vec![method] };

        assert_eq!(r, Done(&b""[..], node));
    }

    #[test]
    fn parse_field_metadata() {
        let res = parse_metadata(b"required ");
        assert_eq!(res, Done(&[][..], FieldMetadataNode::Required));

        let res = parse_metadata(b"optional ");
        assert_eq!(res, Done(&[][..], FieldMetadataNode::Optional));
    }

    #[test]
    fn panic_parse_field_metadata() {
        let res = parse_metadata(b"requiredf ");
        match res {
            Error(_) => {},
            _ => panic!("Unexpected")
        }
    }

    #[test]
    fn define_struct_field() {
        let input = b"1: optional string foobar;";
        let res = parse_struct_field(input);

        let field = StructFieldNode {
            order: 1,
            metadata: FieldMetadataNode::Optional,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };

        assert_eq!(res, Done(&[][..], field));
    }

    #[test]
    fn define_empty_struct() {
        let res = parse_struct(b"struct Foobar {}");
        assert_eq!(res, Done(&[][..], StructNode {
            name: IdentNode(format!("Foobar")),
            fields: vec![]
        }));
    }

    #[test]
    fn define_struct_with_field() {
        let res = parse_struct(b"struct Foobar {
            1: required string name;
        }");

        assert_eq!(res, Done(&[][..], StructNode {
            name: IdentNode(format!("Foobar")),
            fields: vec![
                StructFieldNode {
                    order: 1,
                    metadata: FieldMetadataNode::Required,
                    ty: Ty::String,
                    ident: IdentNode(format!("name"))
                }
            ]
        }));
    }

    #[test]
    fn define_struct_with_multi_fields() {
        let res = parse_struct(b"struct Foobar {
            1: required string name;
            2: optional i64 timestamp;
        }");

        assert_eq!(res, Done(&[][..], StructNode {
            name: IdentNode(format!("Foobar")),
            fields: vec![
                StructFieldNode {
                    order: 1,
                    metadata: FieldMetadataNode::Required,
                    ty: Ty::String,
                    ident: IdentNode(format!("name"))
                },

                StructFieldNode {
                    order: 2,
                    metadata: FieldMetadataNode::Optional,
                    ty: Ty::Signed64,
                    ident: IdentNode(format!("timestamp"))
                }
            ]
        }));
    }

    #[test]
    fn define_service_multi_method() {
        let r = parse_service(b"service Foobar {
            void ping();
            void pong();
        }");

        let ping = FunctionNode { name: IdentNode(format!("ping")), ret: Ty::Void };
        let pong = FunctionNode { name: IdentNode(format!("pong")), ret: Ty::Void };
        let node = ServiceNode { name: IdentNode(format!("Foobar")), methods: vec![ping, pong] };

        assert_eq!(r, Done(&b""[..], node));
    }

    #[test]
    fn gen_ty_string() {
        let mut v = Vec::new();
        let mut s = Ty::String;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "String");
    }

    #[test]
    fn gen_ty_void() {
        let mut v = Vec::new();
        let mut s = Ty::Void;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "()");
    }

    #[test]
    fn gen_ty_bool() {
        let mut v = Vec::new();
        let mut s = Ty::Bool;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "bool");
    }

    #[test]
    fn gen_ty_i16() {
        let mut v = Vec::new();
        let mut s = Ty::Signed16;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "i16");
    }

    #[test]
    fn gen_ty_i32() {
        let mut v = Vec::new();
        let mut s = Ty::Signed32;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "i32");
    }

    #[test]
    fn gen_ty_i64() {
        let mut v = Vec::new();
        let mut s = Ty::Signed64;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "i64");
    }

    #[test]
    fn gen_ty_byte() {
        let mut v = Vec::new();
        let mut s = Ty::Byte;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "i8");
    }

    #[test]
    fn gen_ty_binary() {
        let mut v = Vec::new();
        let mut s = Ty::Binary;
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "Vec<i8>");
    }

    #[test]
    fn gen_ty_list() {
        let mut v = Vec::new();
        let mut s = Ty::List(Box::new(Ty::String));
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "Vec<String>");
    }

    #[test]
    fn gen_ty_map() {
        let mut v = Vec::new();
        let mut s = Ty::Map(Box::new(Ty::String), Box::new(Ty::String));
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "HashMap<String, String>");
    }

    #[test]
    fn gen_struct_field() {
        let mut v = Vec::new();
        let mut s = StructFieldNode {
            order: 1,
            metadata: FieldMetadataNode::Required,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "foobar: String,\n");
    }

    #[test]
    fn gen_struct_optional_field() {
        let mut v = Vec::new();
        let mut s = StructFieldNode {
            order: 1,
            metadata: FieldMetadataNode::Optional,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "foobar: Option<String>,\n");
    }

    #[test]
    fn gen_struct() {
        let mut v = Vec::new();
        let mut field = StructFieldNode {
            order: 1,
            metadata: FieldMetadataNode::Required,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };
        let mut s = StructNode {
            name: IdentNode(format!("Ping")),
            fields: vec![field]
        };

        s.gen(&mut v);

        assert_eq!(str::from_utf8(&v).unwrap(), "pub struct Ping {\nfoobar: String,\n}\n");
    }

    #[test]
    fn gen_struct_multi_fields() {
        let mut v = Vec::new();
        let mut field1 = StructFieldNode {
            order: 1,
            metadata: FieldMetadataNode::Required,
            ty: Ty::Signed64,
            ident: IdentNode(format!("length"))
        };

        let mut field2 = StructFieldNode {
            order: 2,
            metadata: FieldMetadataNode::Required,
            ty: Ty::Binary,
            ident: IdentNode(format!("buffer"))
        };

        let mut s = StructNode {
            name: IdentNode(format!("Data")),
            fields: vec![field1, field2]
        };

        s.gen(&mut v);

        assert_eq!(str::from_utf8(&v).unwrap(), "pub struct Data {\nlength: i64,\nbuffer: Vec<i8>,\n}\n");
    }
}
