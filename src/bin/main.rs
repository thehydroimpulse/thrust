extern crate thrust;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use thrust::Generator;

const USAGE: &'static str = "
Thrust - Thrift for Rust

Usage:
  thrust <input-file> <output-path>
  thrust --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    pub arg_input_file: String,
    pub arg_output_path: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let output = Generator::run(&*args.arg_input_file, &*args.arg_output_path);
    println!("{:?}", output);
}
