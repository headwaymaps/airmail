use std::{fs::File, io::Read};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The model file to use.
    #[clap(long, value_parser)]
    model: String,
    /// The address string to parse.
    #[clap(long, value_parser)]
    str: String,
}

fn main() {
    let args = Args::parse();

    let mut data = vec![];
    File::open(args.model)
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let parser = airmail_lib::parser::Parser::new(&data);
    let parsed = parser.parse(&args.str);

    for parsing in parsed {
        println!("Parsed as: {:?}", parsing);
    }
}
