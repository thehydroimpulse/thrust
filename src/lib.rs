#![feature(type_macros)]

#[macro_use]
extern crate nom;

pub use generator::Generator;
use nom::{IResult};
use std::str;
use std::convert;
use std::string;

mod parser;
mod ast;
mod generator;

pub type ThriftResult<T> = Result<T, ThriftCompilerError>;

#[derive(Debug)]
pub enum ThriftCompilerError {
    Parsing,
    NoNamespace,
    Nom,
    Unknown,
    ToUtf8
}

impl convert::From<string::FromUtf8Error> for ThriftCompilerError {
    fn from(err: string::FromUtf8Error) -> ThriftCompilerError {
        ThriftCompilerError::ToUtf8
    }
}

#[derive(Debug)]
pub struct ThriftCompiler {
    pub namespace: String,
    pub buffer: String
}

impl ThriftCompiler {
    pub fn run(input: &[u8]) -> ThriftResult<ThriftCompiler> {
        match parser::parse_thrift(input) {
            IResult::Done(i, nodes) => {
                let mut buf = Vec::new();
                let mut ns = None;

                for node in nodes.iter() {
                    if node.is_namespace() {
                        ns = node.namespace();
                    } else {
                        node.gen(&mut buf);
                    }
                }

                let ns = match ns {
                    Some(ns) => ns,
                    None => return Err(ThriftCompilerError::NoNamespace)
                };

                return Ok(ThriftCompiler {
                    namespace: ns,
                    buffer: try!(String::from_utf8(buf))
                });
            },
            IResult::Error(err) => {
                return Err(ThriftCompilerError::Nom);
            },
            IResult::Incomplete(n) => {
                println!("{:?}", n);
                return Err(ThriftCompilerError::Unknown);
            }
        }
    }
}
