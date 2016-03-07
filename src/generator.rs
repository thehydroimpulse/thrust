use std::fs::File;
use std::path::Path;
use std::io::{Result, Read, Write};

use ast::Ast;
use parser::parse_thrift;
use nom::IResult;

pub struct Generator {
    input: String,
    output: Option<File>
}

impl Generator {
    pub fn run(path: &str, output: &str) -> &'static str {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => return "Error opening the input .thrift file."
        };

        let mut input = String::new();

        file.read_to_string(&mut input);

        match parse_thrift(&*input.as_bytes()) {
            IResult::Done(i, o) => println!("Done parsing."),
            IResult::Error(err) => println!("{:?}", err),
            IResult::Incomplete(n) => println!("{:?}", n)
        }

        return "Thrust completed successfully!";
    }
}
