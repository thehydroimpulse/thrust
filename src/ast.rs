use std::io::{Read, Write};

pub trait Ast {
    fn gen(&mut self, w: &mut Write) {}
}

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
    pub fn parse(input: String, mut gens: Vec<Ty>) -> Ty {
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

#[derive(PartialEq, Eq, Debug)]
pub struct ServiceNode {
    pub name: IdentNode,
    pub methods: Vec<FunctionNode>
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
    pub name: IdentNode,
    pub ret: Ty
}

#[derive(PartialEq, Eq, Debug)]
pub struct IdentNode(pub String);

#[derive(PartialEq, Eq, Debug)]
pub struct StructNode {
    pub name: IdentNode,
    pub fields: Vec<StructFieldNode>
}

#[derive(PartialEq, Eq, Debug)]
pub enum FieldMetadataNode {
    Required,
    Optional
}

impl FieldMetadataNode {
    pub fn parse(input: &str) -> FieldMetadataNode {
        match input {
            "required" => FieldMetadataNode::Required,
            "optional" => FieldMetadataNode::Optional,
            _ => panic!("invalid field metadata found. Expected `required` or `optional`.")
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct StructFieldNode {
    pub order: u8,
    pub metadata: FieldMetadataNode,
    pub ty: Ty,
    pub ident: IdentNode
}

#[derive(PartialEq, Eq, Debug)]
pub struct EnumNode {
    pub name: IdentNode
}

pub struct NamespaceNode {
    pub lang: IdentNode,
    pub ns: String,
    pub nodes: Vec<Box<Ast>>
}

impl NamespaceNode {
    pub fn new(lang: IdentNode, ns: String) -> NamespaceNode {
        NamespaceNode {
            lang: lang,
            ns: ns,
            nodes: Vec::new()
        }
    }
}

impl Ast for NamespaceNode {
    fn gen(&mut self, w: &mut Write) {
        if &*self.lang.0 == "rust" {
            write!(w, "pub mod {} {{\n", self.ns);

            for node in self.nodes.iter_mut() {
                node.gen(w);
            }

            write!(w, "\n}}");
        }
    }
}
impl Ast for EnumNode {}
impl Ast for IdentNode {}
impl Ast for FunctionNode {}
impl Ast for StructNode {
    fn gen(&mut self, w: &mut Write) {
        write!(w, "pub struct {} {{\n", self.name.0);

        for field in self.fields.iter_mut() {
            field.gen(w);
        }

        write!(w, "}}\n");
    }
}

impl Ast for Ty {
    fn gen(&mut self, w: &mut Write) {
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

impl Ast for StructFieldNode {
    fn gen(&mut self, w: &mut Write) {
        // XXX: Replace `String` with the real type.
        write!(w, "{}: ", self.ident.0);

        if self.metadata == FieldMetadataNode::Optional {
            write!(w, "Option<");
        }

        self.ty.gen(w);

        if self.metadata == FieldMetadataNode::Optional {
            write!(w, ">");
        }

        write!(w, ",\n");
    }
}
