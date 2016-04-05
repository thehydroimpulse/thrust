#![feature(question_mark, quote, rustc_private, associated_type_defaults)]

use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{self, InternedString};
use syntax::ast;
use syntax::ptr::P;

extern crate syntax;

use std::char;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty {
    String,
    Void,
    Byte,
    Bool,
    Binary,
    I16,
    I32,
    I64,
    Double,
    List(Box<Ty>),
    Set(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    Option(Box<Ty>),
    // User-defined type.
    Ident(String)
}

impl From<String> for Ty {
    fn from(val: String) -> Ty {
        match &*val {
            "string" => Ty::String,
            "void" => Ty::Void,
            "byte" => Ty::Byte,
            "bool" => Ty::Bool,
            "binary" => Ty::Binary,
            "i16" => Ty::I16,
            "i32" => Ty::I32,
            "i64" => Ty::I64,
            "double" => Ty::Double,
            _ => Ty::Ident(val)
        }
    }
}

impl Ty {
    pub fn to_ast(&self, cx: &mut ExtCtxt) -> P<ast::Ty> {
        match self {
            &Ty::String => quote_ty!(cx, String),
            &Ty::Void => quote_ty!(cx, ()),
            &Ty::Byte => quote_ty!(cx, i8),
            &Ty::Bool => quote_ty!(cx, bool),
            &Ty::Binary => quote_ty!(cx, Vec<i8>),
            &Ty::I16 => quote_ty!(cx, i16),
            &Ty::I32 => quote_ty!(cx, i32),
            &Ty::I64 => quote_ty!(cx, i64),
            &Ty::Double => quote_ty!(cx, f64),
            &Ty::Option(ref t) => {
                let inner = t.to_ast(cx);
                quote_ty!(cx, Option<$inner>)
            },
            &Ty::List(ref s) => {
                let inner = s.to_ast(cx);
                quote_ty!(cx, Vec<$inner>)
            },
            &Ty::Set(ref s) => {
                let inner = s.to_ast(cx);
                quote_ty!(cx, HashSet<$inner>)
            },
            &Ty::Map(ref a, ref b) => {
                let a = a.to_ast(cx);
                let b = b.to_ast(cx);
                quote_ty!(cx, HashMap<$a, $b>)
            },
            &Ty::Ident(ref s) => {
                let span = cx.call_site();
                cx.ty_ident(span, token::str_to_ident(&s))
            }
        }
    }
}

/// Each argument and return value in Thrift is actually just a struct, which means we need to
/// generate a new one for each of those items.
pub trait IrArgStruct {
    fn ir(&self, cx: &mut ExtCtxt) -> Vec<P<ast::Item>>;
}

/// The Ast is responsible for generic the core items in the tree. These are mostly one-to-one
/// relationships with Rust items. Additional supporting elements are done through later `Ir`
/// traits.
pub trait Ast {
    fn ir(&self, cx: &mut ExtCtxt) -> Option<P<ast::Item>>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Include {
    path: String
}

#[derive(Debug, PartialEq, Eq)]
pub struct Service {
    ident: String,
    methods: Vec<ServiceMethod>
}

impl Ast for Service {
    fn ir(&self, cx: &mut ExtCtxt) -> Option<P<ast::Item>> {
        let span = cx.call_site();
        let mut ident = token::str_to_ident(&self.ident.clone());
        let mut items = Vec::new();

        for method in self.methods.iter() {
            let method_ident = token::str_to_ident(&method.ident);
            let self_ident = token::str_to_ident("self");
            let mut inputs = vec![
                ast::Arg::new_self(span, ast::Mutability::Immutable, self_ident.clone())
            ];
            let ty = method.ty.to_ast(cx);

            for arg in method.args.iter() {
                let arg_ident = token::str_to_ident(&arg.ident);
                let arg_ty = arg.ty.to_ast(cx);
                inputs.push(
                    cx.arg(span, arg_ident, arg_ty)
                );
            }

            let method_node = ast::TraitItemKind::Method(
                ast::MethodSig {
                    unsafety: ast::Unsafety::Normal,
                    constness: ast::Constness::NotConst,
                    abi: syntax::abi::Abi::RustCall,
                    decl: P(ast::FnDecl {
                        inputs: inputs,
                        output: ast::FunctionRetTy::Ty(quote_ty!(cx, tangle::Future<$ty>)),
                        variadic: false
                    }),
                    generics: ast::Generics::default(),
                    explicit_self: ast::ExplicitSelf {
                        node: ast::SelfKind::Region(None, ast::Mutability::Mutable, self_ident),
                        span: span
                    }
                },
                None
            );

            let mut item = ast::TraitItem {
                id: ast::DUMMY_NODE_ID,
                ident: method_ident,
                attrs: Vec::new(),
                node: method_node,
                span: span
            };

            items.push(item);
        }


        let kind = ast::ItemKind::Trait(ast::Unsafety::Normal, ast::Generics::default(), P::new(), items);
        let item = P(ast::Item {
            ident: ident,
            attrs: vec![],
            id: ast::DUMMY_NODE_ID,
            node: kind,
            vis: ast::Visibility::Public,
            span: span
        });

        quote_item!(cx, $item)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceMethod {
    ident: String,
    ty: Ty,
    attr: FieldAttribute,
    args: Vec<StructField>
}

#[derive(Debug, PartialEq, Eq)]
pub struct Enum {
    ident: String,
    variants: Vec<String>
}

impl Ast for Enum {
    fn ir(&self, cx: &mut ExtCtxt) -> Option<P<ast::Item>> {
        let mut ident = token::str_to_ident(&self.ident.clone());
        let mut enum_def = ast::EnumDef {
            variants: Vec::new()
        };

        for node in self.variants.iter() {
            let name = token::str_to_ident(&node);
            let span = cx.call_site();

            enum_def.variants.push(ast::Variant {
                node: ast::Variant_ {
                    name: name,
                    attrs: Vec::new(),
                    data: ast::VariantData::Unit(ast::DUMMY_NODE_ID),
                    disr_expr: None
                },
                span: span
            });
        }

        let span = cx.call_site();
        let derives = vec![
            cx.meta_word(span, InternedString::new("Debug")),
            cx.meta_word(span, InternedString::new("PartialEq")),
            cx.meta_word(span, InternedString::new("Eq")),
            cx.meta_word(span, InternedString::new("Clone")),
            cx.meta_word(span, InternedString::new("Hash")),
        ];
        let attr = ast::Attribute {
            node: ast::Attribute_ {
                id: ast::AttrId(0),
                style: ast::AttrStyle::Inner,
                value: cx.meta_list(span, InternedString::new("derive"), derives),
                is_sugared_doc: false
            },
            span: span
        };

        let kind = ast::ItemKind::Enum(enum_def, ast::Generics::default());
        let item = P(ast::Item {
            ident: ident,
            attrs: vec![attr],
            id: ast::DUMMY_NODE_ID,
            node: kind,
            vis: ast::Visibility::Public,
            span: span
        });

        quote_item!(cx, $item)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Struct {
    ident: String,
    fields: Vec<StructField>
}

impl Ast for Struct {
    fn ir(&self, cx: &mut ExtCtxt) -> Option<P<ast::Item>> {
        let mut ident = token::str_to_ident(&self.ident.clone());
        let mut fields = Vec::new();

        for node in self.fields.iter() {
            let span = cx.call_site();
            let mut ty = node.ty.clone();

            match node.attr {
                FieldAttribute::Required => {},
                // XXX: We need to map the inner `node.ty` to a proper Rust type.
                FieldAttribute::Optional => ty = Ty::Option(Box::new(ty)),
                _ => panic!("Oneway is not supported for struct fields.")
            }

            let field = ast::StructField {
                node: ast::StructField_ {
                    kind: ast::StructFieldKind::NamedField(token::str_to_ident(&node.ident), ast::Visibility::Public),
                    id: ast::DUMMY_NODE_ID,
                    ty: ty.to_ast(cx),
                    attrs: Vec::new()
                },
                span: span
            };
            fields.push(field);
        }

        let span = cx.call_site();
        let derives = vec![
            cx.meta_word(span, InternedString::new("Debug")),
            cx.meta_word(span, InternedString::new("PartialEq")),
            cx.meta_word(span, InternedString::new("Eq")),
            cx.meta_word(span, InternedString::new("Clone")),
            cx.meta_word(span, InternedString::new("Hash")),
        ];
        let attr = ast::Attribute {
            node: ast::Attribute_ {
                id: ast::AttrId(0),
                style: ast::AttrStyle::Inner,
                value: cx.meta_list(span, InternedString::new("derive"), derives),
                is_sugared_doc: false
            },
            span: span
        };
        let struct_def = ast::VariantData::Struct(fields, ast::DUMMY_NODE_ID);
        let kind = ast::ItemKind::Struct(struct_def, ast::Generics::default());

        let item = P(ast::Item {
            ident: ident,
            attrs: vec![attr],
            id: ast::DUMMY_NODE_ID,
            node: kind,
            vis: ast::Visibility::Public,
            span: span
        });

        quote_item!(cx, $item)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldAttribute {
    Optional,
    Required,
    Oneway
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructField {
    seq: i16,
    attr: FieldAttribute,
    ty: Ty,
    ident: String
}

#[derive(Debug, PartialEq, Eq)]
pub struct Typedef(pub Ty, pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct Namespace {
    pub lang: String,
    pub module: String
}

impl Ast for Namespace {
    fn ir(&self, cx: &mut ExtCtxt) -> Option<P<ast::Item>> {
        None
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Keyword {
    Struct,
    Service,
    Enum,
    Namespace,
    Required,
    Optional,
    Oneway,
    Typedef,
    Throws,
    Exception,
    Include,
    Const,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    Eq,
    Colon,
    SingleQuote,
    Dot,
    Semi,
    At,
    Comma,
    LCurly,
    RCurly,
    LAngle,
    RAngle,
    LParen,
    RParen,
    Number(i16),
    QuotedString(String),
    Ident(String),
    Keyword(Keyword),

    /// Useless comments.
    Comment,
    Whitespace,
    Eof,
    B,
}


fn map_ty(ty: &str) -> ast::Ident {
    let ty = match ty {
        "string" => "String",
        "byte" => "i8",
        "bool" => "bool",
        "i16" => "i16",
        "i32" => "i32",
        "i64" => "i64",
        "double" => "f64",
        "binary" => "Vec<i8>",
        s => s
    };

    token::str_to_ident(ty)
}


#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Expected,
    MissingFieldAttribute,
    ExpectedNumber,
    ExpectedString,
    ExpectedKeyword(Keyword),
    ExpectedIdent,
    ExpectedToken(Token),
    NoMoreItems
}

pub struct Parser<'a> {
    buffer: &'a str,
    pos: usize,
    token: Token,
    last_token_eof: bool
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            buffer: input,
            pos: 0,
            token: Token::B,
            last_token_eof: false
        }
    }

    pub fn parse_struct(&mut self) -> Result<Struct, Error> {
        self.expect_keyword(Keyword::Struct)?;

        let ident = self.expect_ident()?;
        let mut fields = Vec::new();

        self.expect(&Token::LCurly)?;

        loop {
            if self.eat(&Token::RCurly) {
                break;
            }

            fields.push(self.parse_struct_field()?);

            if self.eat(&Token::Semi) {
                continue;
            } else {
                break;
            }
        }

        Ok(Struct {
            ident: ident,
            fields: fields
        })
    }

    pub fn parse_struct_field(&mut self) -> Result<StructField, Error> {
        let seq = self.parse_number()?;

        self.expect(&Token::Colon)?;

        let attr = if self.eat_keyword(Keyword::Optional) {
            FieldAttribute::Optional
        } else if self.eat_keyword(Keyword::Required) {
            FieldAttribute::Required
        } else {
            return Err(Error::MissingFieldAttribute);
        };

        let ty = self.parse_ty()?;
        let ident = self.parse_ident()?;

        Ok(StructField {
            seq: seq,
            attr: attr,
            ty: ty,
            ident: ident
        })
    }

    pub fn parse_number(&mut self) -> Result<i16, Error> {
        self.skip_b();

        let n = match self.token {
            Token::Number(n) => n,
            _ => return Err(Error::ExpectedNumber)
        };

        self.bump();
        Ok(n)
    }

    pub fn skip_b(&mut self) {
        if self.token == Token::B {
            self.bump();
        }
    }

    pub fn parse_enum(&mut self) -> Result<Enum, Error> {
        self.expect_keyword(Keyword::Enum)?;

        let ident = self.expect_ident()?;
        let mut variants = Vec::new();

        self.expect(&Token::LCurly)?;

        loop {
            if self.eat(&Token::RCurly) {
                break;
            }

            variants.push(self.parse_ident()?);

            if self.eat(&Token::Comma) {
                continue;
            } else {
                self.eat(&Token::RCurly);
                break;
            }
        }

        Ok(Enum {
            ident: ident,
            variants: variants
        })
    }

    pub fn parse_include(&mut self) -> Result<Include, Error> {
        self.expect_keyword(Keyword::Include)?;

        Ok(Include {
            path: self.expect_string()?
        })
    }

    pub fn parse_typedef(&mut self) -> Result<Typedef, Error> {
        self.expect_keyword(Keyword::Typedef)?;

        Ok(Typedef(self.parse_ty()?, self.expect_ident()?))
    }

    pub fn parse_namespace(&mut self) -> Result<Namespace, Error> {
        self.expect_keyword(Keyword::Namespace)?;

        let lang = self.expect_ident()?;
        let module = self.expect_ident()?;

        Ok(Namespace {
            lang: lang,
            module: module
        })
    }

    pub fn parse_service(&mut self) -> Result<Service, Error> {
        self.expect_keyword(Keyword::Service)?;

        let mut methods = Vec::new();
        let ident = self.expect_ident()?;
        self.expect(&Token::LCurly)?;

        loop {
            if self.eat(&Token::RCurly) {
                break;
            }

            // Try and eat a keyword
            let method_attr = if self.eat_keyword(Keyword::Oneway) {
                FieldAttribute::Oneway
            } else {
                // This is mostly ignored, we just need some sort of value here.
                FieldAttribute::Required
            };

            let method_ty = self.parse_ty()?;
            let method_ident = self.parse_ident()?;
            let mut method_fields = Vec::new();

            self.expect(&Token::LParen)?;

            loop {
                if self.eat(&Token::RParen) {
                    break;
                }

                let seq = self.parse_number()?;
                self.expect(&Token::Colon)?;
                let field_ty = self.parse_ty()?;
                let field_ident = self.parse_ident()?;

                method_fields.push(StructField {
                    seq: seq,
                    attr: FieldAttribute::Required,
                    ty: field_ty,
                    ident: field_ident
                });

                if self.eat(&Token::Comma) {
                    continue;
                } else if self.eat(&Token::RParen) {
                    break;
                } else {
                    panic!("failed to properly parse the service {:?}", ident);
                }
            }

            methods.push(ServiceMethod {
                ident: method_ident,
                ty: method_ty,
                attr: method_attr,
                args: method_fields
            });

            if self.eat(&Token::Comma) || self.eat(&Token::Semi) {
                continue;
            } else {
                self.eat(&Token::RCurly);
                break;
            }
        }

        Ok(Service {
            ident: ident,
            methods: methods
        })
    }

    pub fn expect_string(&mut self) -> Result<String, Error> {
        let val = match self.token {
            Token::QuotedString(ref s) => s.clone(),
            _ => return Err(Error::ExpectedString)
        };

        self.bump();
        Ok(val)
    }

    pub fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), Error> {
        if !self.eat_keyword(keyword) {
            return Err(Error::ExpectedKeyword(keyword));
        }

        Ok(())
    }

    pub fn expect(&mut self, token: &Token) -> Result<Token, Error> {
        if !self.eat(token) {
            return Err(Error::ExpectedToken(token.clone()));
        } else {
            Ok(self.token.clone())
        }
    }

    pub fn parse_ident(&mut self) -> Result<String, Error> {
        if self.token == Token::B {
            self.bump();
        }

        let i = match self.token {
            Token::Ident(ref s) => s.clone(),
            _ => return Err(Error::ExpectedIdent)
        };

        self.bump();
        Ok(i)
    }

    pub fn parse_ty(&mut self) -> Result<Ty, Error> {
        let ident = self.parse_ident()?;
        // map, set, list
        if self.eat(&Token::LAngle) {
            let ty = match &*ident {
                "map" => {
                    let a = self.parse_ty()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_ty()?;

                    Ty::Map(Box::new(a), Box::new(b))
                },
                "set" => Ty::Set(Box::new(self.parse_ty()?)),
                "list" => Ty::List(Box::new(self.parse_ty()?)),
                _ => panic!("Error!")
            };

            self.expect(&Token::RAngle)?;

            Ok(ty)
        } else {
            Ok(Ty::from(ident))
        }
    }

    pub fn expect_ident(&mut self) -> Result<String, Error> {
        let ident = match self.token {
            Token::Ident(ref s) => s.clone(),
            _ => return Err(Error::Expected)
        };

        self.bump();
        Ok(ident)
    }

    pub fn parse_item(&mut self) -> Result<Box<Ast>, Error> {
        if self.lookahead_keyword(Keyword::Namespace) {
            Ok(Box::new(self.parse_namespace()?))
        } else if self.lookahead_keyword(Keyword::Enum) {
            Ok(Box::new(self.parse_enum()?))
        } else if self.lookahead_keyword(Keyword::Struct) {
            Ok(Box::new(self.parse_struct()?))
        } else if self.lookahead_keyword(Keyword::Service) {
            Ok(Box::new(self.parse_service()?))
        } else {
            Err(Error::NoMoreItems)
        }
    }

    pub fn lookahead_keyword(&mut self, keyword: Keyword) -> bool {
        self.lookahead(&Token::Keyword(keyword))
    }

    pub fn lookahead(&mut self, token: &Token) -> bool {
        if self.token == *token {
            true
        } else {
            false
        }
    }

    pub fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        self.eat(&Token::Keyword(keyword))
    }

    fn next_char(&self) -> char {
        self.buffer[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.buffer[self.pos ..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.buffer.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.buffer[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        return cur_char;
    }

    fn next_token(&mut self) -> Token {
        if self.eof() {
            return Token::Eof;
        }

        let ch = self.consume_char();

        match ch {
            ':' => Token::Colon,
            '.' => Token::Dot,
            ';' => Token::Semi,
            ',' => Token::Comma,
            '"' => {
                let val = self.consume_while(|c| c != '"' || c != '\"');
                self.consume_char();
                Token::QuotedString(val)
            },
            '=' => Token::Eq,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LCurly,
            '}' => Token::RCurly,
            '<' => Token::LAngle,
            '>' => Token::RAngle,
            '0'...'9' => {
                let mut val = self.consume_while(|c| match c {
                    '0'...'9' => true,
                    _ => false
                });

                val = format!("{}{}", ch, val);

                Token::Number(val.parse().unwrap())
            },
            '/' | '#' => {
                if self.next_char() == '/' || ch == '#' {
                    self.consume_while(|c| c != '\n' && c != '\r');
                    return Token::Comment
                } else if self.next_char() == '*' {
                    self.consume_char();
                    loop {
                        self.consume_while(|c| c != '*');
                        self.consume_char();

                        if self.next_char() == '/' {
                            break;
                        }
                    }

                    // Consume the following '/' because we just did a lookahead previously.
                    self.consume_char();
                    return Token::Comment
                }

                Token::Eof
            },
            c if c.is_whitespace() => {
                self.consume_whitespace();
                self.next_token()
            },
            // identifier
            'a'...'z' | 'A'...'Z' | '_' => {
                let mut ident = self.consume_while(|c| match c {
                    'a'...'z' => true,
                    'A'...'Z' => true,
                    '_' => true,
                    '0'...'9' => true,
                    _ => false
                });

                ident = format!("{}{}", ch, ident);

                match &*ident {
                    "namespace" => return Token::Keyword(Keyword::Namespace),
                    "struct" => return Token::Keyword(Keyword::Struct),
                    "enum" => return Token::Keyword(Keyword::Enum),
                    "service" => return Token::Keyword(Keyword::Service),
                    "optional" => return Token::Keyword(Keyword::Optional),
                    "required" => return Token::Keyword(Keyword::Required),
                    "throws" => return Token::Keyword(Keyword::Throws),
                    "oneway" => return Token::Keyword(Keyword::Oneway),
                    "typedef" => return Token::Keyword(Keyword::Typedef),
                    "exception" => return Token::Keyword(Keyword::Exception),
                    "include" => return Token::Keyword(Keyword::Include),
                    "const" => return Token::Keyword(Keyword::Const),
                    _ => Token::Ident(ident)
                }
            },
            _ => Token::Eof
        }
    }

    pub fn eat(&mut self, token: &Token) -> bool {
        if self.token == Token::B {
            self.bump();
        }

        if self.token == *token {
            self.bump();
            true
        } else {
            false
        }
    }

    fn consume_while<F>(&mut self, test: F) -> String
        where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        return result;
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    pub fn bump(&mut self) {
        if self.last_token_eof {
            panic!("attempted to bump past eof.");
        }

        if self.token == Token::Eof {
            self.last_token_eof = true;
        }

        self.token = self.next_token();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eof_token() {
        let mut parser = Parser::new("");
        assert_eq!(parser.eof(), true);
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn colon_token() {
        let mut parser = Parser::new(":");
        assert_eq!(parser.next_token(), Token::Colon);
    }

    #[test]
    fn dot_token() {
        let mut parser = Parser::new(".");
        assert_eq!(parser.next_token(), Token::Dot);
    }

    #[test]
    fn equal_token() {
        let mut parser = Parser::new("=");
        assert_eq!(parser.next_token(), Token::Eq);
    }

    #[test]
    fn comma_token() {
        let mut parser = Parser::new(",");
        assert_eq!(parser.next_token(), Token::Comma);
    }

    #[test]
    fn curly_token() {
        let mut parser = Parser::new("{}");
        assert_eq!(parser.next_token(), Token::LCurly);
        assert_eq!(parser.next_token(), Token::RCurly);
    }

    #[test]
    fn angle_token() {
        let mut parser = Parser::new("<>");
        assert_eq!(parser.next_token(), Token::LAngle);
        assert_eq!(parser.next_token(), Token::RAngle);
    }

    #[test]
    fn semi_token() {
        let mut parser = Parser::new(";");
        assert_eq!(parser.next_token(), Token::Semi);
    }

    #[test]
    #[should_panic]
    fn whitespace_token() {
        let mut parser = Parser::new(" ");
        assert_eq!(parser.next_token(), Token::Whitespace);
    }

    #[test]
    fn whitespace_grab_token() {
        let mut parser = Parser::new("     >");
        assert_eq!(parser.next_token(), Token::RAngle);
    }

    #[test]
    fn comment_token() {
        let mut parser = Parser::new("<//foobar\n:");
        assert_eq!(parser.next_token(), Token::LAngle);
        assert_eq!(parser.next_token(), Token::Comment);
        assert_eq!(parser.next_token(), Token::Colon);
    }

    #[test]
    fn multi_comment_token() {
        let mut parser = Parser::new("</*\n
                                     fofoo*/>");
        assert_eq!(parser.next_token(), Token::LAngle);
        assert_eq!(parser.next_token(), Token::Comment);
        assert_eq!(parser.next_token(), Token::RAngle);
    }

    #[test]
    fn parens_token() {
        let mut parser = Parser::new("()");
        assert_eq!(parser.next_token(), Token::LParen);
        assert_eq!(parser.next_token(), Token::RParen);
    }

    #[test]
    fn hash_comment_token() {
        let mut parser = Parser::new("<#foobar\n:");
        assert_eq!(parser.next_token(), Token::LAngle);
        assert_eq!(parser.next_token(), Token::Comment);
        assert_eq!(parser.next_token(), Token::Colon);
    }

    #[test]
    fn ident_token() {
        assert_eq!(Parser::new("foobar").next_token(), Token::Ident("foobar".to_string()));
        assert_eq!(Parser::new("foobar123").next_token(), Token::Ident("foobar123".to_string()));
        assert_eq!(Parser::new("foobar_123").next_token(), Token::Ident("foobar_123".to_string()));
        assert_eq!(Parser::new("_FFF").next_token(), Token::Ident("_FFF".to_string()));
    }

    #[test]
    fn quoted_string_token() {
        assert_eq!(Parser::new("\"hello world 12338383\"").next_token(), Token::QuotedString("hello world 12338383".to_string()));
    }

    #[test]
    #[should_panic]
    fn fail_ident_token() {
        assert_eq!(Parser::new("1foobar").next_token(), Token::Ident("1foobar".to_string()));
    }

    #[test]
    fn keywords_token() {
        assert_eq!(Parser::new("oneway").next_token(), Token::Keyword(Keyword::Oneway));
        assert_eq!(Parser::new("exception").next_token(), Token::Keyword(Keyword::Exception));
        assert_eq!(Parser::new("struct").next_token(), Token::Keyword(Keyword::Struct));
        assert_eq!(Parser::new("enum").next_token(), Token::Keyword(Keyword::Enum));
        assert_eq!(Parser::new("namespace").next_token(), Token::Keyword(Keyword::Namespace));
        assert_eq!(Parser::new("service").next_token(), Token::Keyword(Keyword::Service));
        assert_eq!(Parser::new("throws").next_token(), Token::Keyword(Keyword::Throws));
        assert_eq!(Parser::new("typedef").next_token(), Token::Keyword(Keyword::Typedef));
        assert_eq!(Parser::new("optional").next_token(), Token::Keyword(Keyword::Optional));
        assert_eq!(Parser::new("required").next_token(), Token::Keyword(Keyword::Required));
        assert_eq!(Parser::new("const").next_token(), Token::Keyword(Keyword::Const));
    }

    #[test]
    fn eat_token() {
        let mut p = Parser::new(":");
        assert_eq!(p.eat(&Token::Colon), true);
    }

    #[test]
    fn eat_keywords() {
        let mut p = Parser::new("oneway");
        assert_eq!(p.eat_keyword(Keyword::Oneway), true);
    }

    #[test]
    fn parse_namespace() {
        let mut p = Parser::new("namespace rust foobar");
        let ns = p.parse_namespace().unwrap();
        assert_eq!(&*ns.lang, "rust");
        assert_eq!(&*ns.module, "foobar");
    }

    #[test]
    fn parse_namesp_ace() {
        let mut p = Parser::new("namespace rust foobar");
        let ns = p.parse_namespace().unwrap();
        assert_eq!(&*ns.lang, "rust");
        assert_eq!(&*ns.module, "foobar");
    }

    #[test]
    fn parse_include() {
        let mut p = Parser::new("include \"./../include.thrift\"");
        let ns = p.parse_include().unwrap();
        assert_eq!(&*ns.path, "./../include.thrift");
    }

    #[test]
    fn parse_bool_ty() {
        let mut p = Parser::new("bool");
        assert_eq!(p.parse_ty().unwrap(), Ty::Bool);
    }

    #[test]
    fn parse_binary_ty() {
        let mut p = Parser::new("binary");
        assert_eq!(p.parse_ty().unwrap(), Ty::Binary);
    }

    #[test]
    fn parse_byte_ty() {
        let mut p = Parser::new("byte");
        assert_eq!(p.parse_ty().unwrap(), Ty::Byte);
    }

    #[test]
    fn parse_i16_ty() {
        let mut p = Parser::new("i16");
        assert_eq!(p.parse_ty().unwrap(), Ty::I16);
    }

    #[test]
    fn parse_i32_ty() {
        let mut p = Parser::new("i32");
        assert_eq!(p.parse_ty().unwrap(), Ty::I32);
    }

    #[test]
    fn parse_i64_ty() {
        let mut p = Parser::new("i64");
        assert_eq!(p.parse_ty().unwrap(), Ty::I64);
    }

    #[test]
    fn parse_double_ty() {
        let mut p = Parser::new("double");
        assert_eq!(p.parse_ty().unwrap(), Ty::Double);
    }

    #[test]
    fn parse_string_ty() {
        let mut p = Parser::new("string");
        assert_eq!(p.parse_ty().unwrap(), Ty::String);
    }

    #[test]
    fn parse_list_string_ty() {
        let mut p = Parser::new("list<string>");
        assert_eq!(p.parse_ty().unwrap(), Ty::List(Box::new(Ty::String)));
    }

    #[test]
    fn parse_list_double_ty() {
        let mut p = Parser::new("list<double>");
        assert_eq!(p.parse_ty().unwrap(), Ty::List(Box::new(Ty::Double)));
    }

    #[test]
    fn parse_list_list_byte_ty() {
        let mut p = Parser::new("list<list<byte>>");
        assert_eq!(p.parse_ty().unwrap(), Ty::List(Box::new(Ty::List(Box::new(Ty::Byte)))));
    }

    #[test]
    fn parse_set_byte_ty() {
        let mut p = Parser::new("set<byte>");
        assert_eq!(p.parse_ty().unwrap(), Ty::Set(Box::new(Ty::Byte)));
    }

    #[test]
    fn parse_set_string_ty() {
        let mut p = Parser::new("set<string>");
        assert_eq!(p.parse_ty().unwrap(), Ty::Set(Box::new(Ty::String)));
    }

    #[test]
    fn parse_map_i32_string_ty() {
        let mut p = Parser::new("map<i32,string>");
        assert_eq!(p.parse_ty().unwrap(), Ty::Map(Box::new(Ty::I32), Box::new(Ty::String)));
    }

    #[test]
    fn parse_map_i32_list_string_ty() {
        let mut p = Parser::new("map<i32,list<string>>");
        assert_eq!(p.parse_ty().unwrap(), Ty::Map(Box::new(Ty::I32), Box::new(Ty::List(Box::new(Ty::String)))));
    }

    #[test]
    fn parse_typedef() {
        let mut p = Parser::new("typedef i32 MyInteger");
        let def = p.parse_typedef().unwrap();
        assert_eq!(def.0, Ty::I32);
        assert_eq!(&*def.1, "MyInteger");
    }

    #[test]
    fn parse_empty_enum() {
        let mut p = Parser::new("enum FooBar {}");
        let def = p.parse_enum().unwrap();
        assert_eq!(&*def.ident, "FooBar");
        assert_eq!(def.variants.len(), 0);
    }

    #[test]
    fn parse_one_variant_enum() {
        let mut p = Parser::new("enum Hello { ONE }");
        let def = p.parse_enum().unwrap();
        assert_eq!(&*def.ident, "Hello");
        assert_eq!(def.variants.len(), 1);
        assert_eq!(&*def.variants[0], "ONE");
    }

    #[test]
    fn parse_empty_service() {
        let mut p = Parser::new("service Flock {}");
        let def = p.parse_service().unwrap();
        assert_eq!(&*def.ident, "Flock");
        assert_eq!(def.methods.len(), 0);
    }

    #[test]
    fn parse_method_service() {
        let mut p = Parser::new("service Flock {
                                    void ping();
                                }");
        let def = p.parse_service().unwrap();
        assert_eq!(&*def.ident, "Flock");
        assert_eq!(def.methods.len(), 1);
        assert_eq!(&*def.methods[0].ident, "ping");
        assert_eq!(def.methods[0].ty, Ty::Void);
        assert_eq!(def.methods[0].attr, FieldAttribute::Required);
        assert_eq!(def.methods[0].args.len(), 0);
    }

    #[test]
    fn parse_method_with_one_args_service() {
        let mut p = Parser::new("service Beans {
                                    void poutine(1: string firstName);
                                }");
        let def = p.parse_service().unwrap();
        assert_eq!(&*def.ident, "Beans");
        assert_eq!(def.methods.len(), 1);
        assert_eq!(&*def.methods[0].ident, "poutine");
        assert_eq!(def.methods[0].ty, Ty::Void);
        assert_eq!(def.methods[0].attr, FieldAttribute::Required);
        assert_eq!(def.methods[0].args.len(), 1);
        assert_eq!(def.methods[0].args[0], StructField {
            seq: 1,
            attr: FieldAttribute::Required,
            ty: Ty::String,
            ident: "firstName".to_string()
        });
    }

    #[test]
    fn parse_oneway_method_service() {
        let mut p = Parser::new("service Flock {
                                    oneway void ping();
                                }");
        let def = p.parse_service().unwrap();
        assert_eq!(&*def.ident, "Flock");
        assert_eq!(def.methods.len(), 1);
        assert_eq!(&*def.methods[0].ident, "ping");
        assert!(def.methods[0].ty == Ty::Void);
        assert_eq!(def.methods[0].attr, FieldAttribute::Oneway);
        assert_eq!(def.methods[0].args.len(), 0);
    }

    #[test]
    fn parse_multi_variant_enum() {
        let mut p = Parser::new("enum Hello { ONE, TWO }");
        let def = p.parse_enum().unwrap();
        assert_eq!(&*def.ident, "Hello");
        assert_eq!(def.variants.len(), 2);
        assert_eq!(&*def.variants[0], "ONE");
        assert_eq!(&*def.variants[1], "TWO");
    }

    #[test]
    fn parse_empty_struct() {
        let mut p = Parser::new("struct FooBar {}");
        let def = p.parse_struct().unwrap();
        assert_eq!(&*def.ident, "FooBar");
        assert_eq!(def.fields.len(), 0);
    }

    #[test]
    fn parse_struct_w_field() {
        let mut p = Parser::new("struct FooBar { 1: required i32 mycat }");
        let def = p.parse_struct().unwrap();
        assert_eq!(&*def.ident, "FooBar");
        assert_eq!(def.fields.len(), 1);
    }

    #[test]
    fn parse_struct_w_multi_field() {
        let mut p = Parser::new("struct FooBar { 1: required i32 mycat; 2: required i32 two }");
        let def = p.parse_struct().unwrap();
        assert_eq!(&*def.ident, "FooBar");
        assert_eq!(def.fields.len(), 2);
    }

    #[test]
    fn parse_struct_field_optional() {
        let mut p = Parser::new("1: optional i32 foobar");
        let def = p.parse_struct_field().unwrap();
        assert_eq!(&*def.ident, "foobar");
        assert_eq!(def.ty, Ty::I32);
        assert_eq!(def.seq, 1);
        assert_eq!(def.attr, FieldAttribute::Optional);
    }

    #[test]
    fn parse_struct_field_required() {
        let mut p = Parser::new("1: required i32 foobar");
        let def = p.parse_struct_field().unwrap();
        assert_eq!(&*def.ident, "foobar");
        assert_eq!(def.ty, Ty::I32);
        assert_eq!(def.seq, 1);
        assert_eq!(def.attr, FieldAttribute::Required);
    }
}
