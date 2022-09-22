use std::{fs::File, io::Read};

use airmail_util::model::Model;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The model file to dump.
    #[clap(long, value_parser)]
    model: String,
    /// The model file to dump.
    #[clap(long, value_parser)]
    packed: String,
}

fn main() {
    let args = Args::parse();
    if args.packed == args.model {
        panic!();
    }

    let mut model_data = vec![];
    File::open(args.model)
        .unwrap()
        .read_to_end(&mut model_data)
        .unwrap();
    let model = Model::new(&model_data).unwrap();
    let mut packed_file = File::create(args.packed).unwrap();
    model.dump(&mut packed_file).unwrap();
}
