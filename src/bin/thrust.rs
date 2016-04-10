extern crate thrust_codegen;
extern crate thrust_parser;
extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

use std::io::{Write, Read};
use std::fs::File;
use std::path::Path;
use thrust_parser::Parser;
use thrust_codegen::{compile, find_rust_namespace};

const USAGE: &'static str = "
Thrust: Thrift compiler for Rust

Usage:
  thrust <input> <output>
  thrust --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_input: String,
    arg_output: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    let mut input = File::open(args.arg_input).expect("input file does not exist.");
    let mut s = String::new();
    input.read_to_string(&mut s).unwrap();
    let mut parser = Parser::new(&s);
    let ns = find_rust_namespace(&mut parser).unwrap();

    let module = Path::new(&args.arg_output).join(ns.module).with_extension("rs");
    let mut output = File::create(module).expect("error creating the module.");

    compile(&mut parser, &mut output).unwrap();
    // println!("{}", String::from_utf8(buf).unwrap());
    // let mut file = File::create("src/testing.rs").unwrap();
    // file.write_all(&buf[..]).unwrap();
}
