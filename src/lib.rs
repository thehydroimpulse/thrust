use nom::{space, alphanumeric, IResult, multispace, not_line_ending, Err, be_u8, digit};
use nom::IResult::*;
use std::str;
use std::convert::From;
use std::io::Write;

#[macro_use]
extern crate nom;

#[derive(PartialEq, Eq, Debug)]
pub enum Ty {
    Void,
    Bool,
    Byte,
    Signed16,
    Signed32,
    Signed64,
    Double,
    Binary,
    String,
    List(Box<Ty>),
    Set(Box<Ty>),
    Map(Box<Ty>, Box<Ty>)
}

impl Ty {
    fn parse(input: String, mut gens: Vec<Ty>) -> Ty {
        match &*input {
            "byte" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `byte`");
                }

                Ty::Byte
            },
            "void" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `void`");
                }

                Ty::Void
            },
            "bool" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `bool`");
                }

                Ty::Bool
            },
            "i16" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `i16`");
                }

                Ty::Signed16
            },
            "i32" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `i32`");
                }

                Ty::Signed32
            },
            "i64" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `i64`");
                }

                Ty::Signed64
            },
            "double" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `double`");
                }

                Ty::Double
            },
            "binary" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `binary`");
                }

                Ty::Binary
            },
            "string" => {
                if gens.len() != 0 {
                    panic!("unexpected generic for `string`");
                }

                Ty::String
            },
            "list" => {
                if gens.len() != 1 {
                    panic!("Expected type argument to `list`.");
                }

                Ty::List(Box::new(gens.pop().unwrap()))
            },
            "set" => {
                if gens.len() != 1 {
                    panic!("Expected type argument to `set`.");
                }

                Ty::Set(Box::new(gens.pop().unwrap()))
            },
            "map" => {
                if gens.len() != 2 {
                    panic!("Expected 2 type argument to `map`.");
                }

                let last = gens.pop().unwrap();
                let first = gens.pop().unwrap();

                Ty::Map(Box::new(first), Box::new(last))
            },
            _ => panic!("Unexpected type {:?}. Expected a type at that position.", input)
        }
    }
}

pub trait Ast {
    fn gen<W: Write>(&mut self, w: &mut W) {}
}

#[derive(PartialEq, Eq, Debug)]
pub struct ServiceNode {
    name: IdentNode,
    methods: Vec<FunctionNode>
}

impl ServiceNode {
    pub fn new(name: IdentNode) -> ServiceNode {
        ServiceNode {
            name: name,
            methods: Vec::new()
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct FunctionNode {
    name: IdentNode,
    ret: Ty
}

#[derive(PartialEq, Eq, Debug)]
pub struct IdentNode(String);

#[derive(PartialEq, Eq, Debug)]
pub struct StructNode {
    name: IdentNode,
    fields: Vec<StructField>
}

#[derive(PartialEq, Eq, Debug)]
pub enum FieldMetadata {
    Required,
    Optional
}

impl FieldMetadata {
    pub fn parse(input: &str) -> FieldMetadata {
        match input {
            "required" => FieldMetadata::Required,
            "optional" => FieldMetadata::Optional,
            _ => panic!("invalid field metadata found. Expected `required` or `optional`.")
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct StructField {
    order: u8,
    metadata: FieldMetadata,
    ty: Ty,
    ident: IdentNode
}

#[derive(PartialEq, Eq, Debug)]
pub struct EnumNode {
    name: IdentNode
}

impl Ast for EnumNode {}
impl Ast for IdentNode {}
impl Ast for FunctionNode {}
impl Ast for StructNode {
    fn gen<W: Write>(&mut self, w: &mut W) {
        write!(w, "pub struct {} {{\n", self.name.0);

        for field in self.fields.iter_mut() {
            field.gen(w);
        }

        write!(w, "}}\n");
    }
}

impl Ast for Ty {
    fn gen<W: Write>(&mut self, w: &mut W) {
        match self {
            &mut Ty::String => { write!(w, "String"); },
            &mut Ty::Void => { write!(w, "()"); },
            &mut Ty::Byte => { write!(w, "i8"); },
            &mut Ty::Binary => { write!(w, "Vec<i8>"); },
            &mut Ty::Signed16 => { write!(w, "i16"); },
            &mut Ty::Signed32 => { write!(w, "i32"); },
            &mut Ty::Signed64 => { write!(w, "i64"); },
            &mut Ty::Bool => { write!(w, "bool"); },
            &mut Ty::List(ref mut t) => {
                write!(w, "Vec<");
                t.gen(w);
                write!(w, ">");
            },
            &mut Ty::Map(ref mut k, ref mut v) => {
                write!(w, "HashMap<");
                k.gen(w);
                write!(w, ", ");
                v.gen(w);
                write!(w, ">");
            },
            _ => {}
        }
    }
}

impl Ast for StructField {
    fn gen<W: Write>(&mut self, w: &mut W) {
        // XXX: Replace `String` with the real type.
        write!(w, "{}: ", self.ident.0);

        if self.metadata == FieldMetadata::Optional {
            write!(w, "Option<");
        }

        self.ty.gen(w);

        if self.metadata == FieldMetadata::Optional {
            write!(w, ">");
        }

        write!(w, ",\n");
    }
}

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
        Error(Err::Code(0))
    }
}

named!(pub parse_ident<&[u8], &str>, map_res!(alphanumeric, str::from_utf8));

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

named!(pub parse_metadata<&[u8], FieldMetadata>, chain!(
    meta: map_res!(alt!(tag!("optional") | tag!("required")), str::from_utf8) ~
    space,
    || { FieldMetadata::parse(meta) }
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

named!(pub parse_struct_field<&[u8], StructField>, chain!(
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
        StructField {
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
        let res = parse_metadata(b"required");
        assert_eq!(res, Done(&[][..], FieldMetadata::Required));

        let res = parse_metadata(b"optional");
        assert_eq!(res, Done(&[][..], FieldMetadata::Optional));
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

        let field = StructField {
            order: 1,
            metadata: FieldMetadata::Optional,
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
                StructField {
                    order: 1,
                    metadata: FieldMetadata::Required,
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
                StructField {
                    order: 1,
                    metadata: FieldMetadata::Required,
                    ty: Ty::String,
                    ident: IdentNode(format!("name"))
                },

                StructField {
                    order: 2,
                    metadata: FieldMetadata::Optional,
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
        let mut s = StructField {
            order: 1,
            metadata: FieldMetadata::Required,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "foobar: String,\n");
    }

    #[test]
    fn gen_struct_optional_field() {
        let mut v = Vec::new();
        let mut s = StructField {
            order: 1,
            metadata: FieldMetadata::Optional,
            ty: Ty::String,
            ident: IdentNode(format!("foobar"))
        };
        s.gen(&mut v);
        assert_eq!(str::from_utf8(&v).unwrap(), "foobar: Option<String>,\n");
    }

    #[test]
    fn gen_struct() {
        let mut v = Vec::new();
        let mut field = StructField {
            order: 1,
            metadata: FieldMetadata::Required,
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
}
