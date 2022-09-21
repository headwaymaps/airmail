use std::{fs::File, io::Read};

use airmail_lib::tokenizer::Tokenizer;
use clap::Parser;
use fst::{raw::Fst, Streamer};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The vocabulary file to use.
    #[clap(long, value_parser)]
    vocab: String,
    /// The string to tokenize.
    #[clap(long, value_parser)]
    str: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut vocab_data = vec![];
    File::open(args.vocab)
        .unwrap()
        .read_to_end(&mut vocab_data)
        .unwrap();

    if let Some(string) = args.str {
        let fst = Fst::new(vocab_data).unwrap();
        let tokenizer = Tokenizer::new(&fst);
        let features = tokenizer.tokenize(&string);
        for word_features in features {
            let mut word_feature_strings: Vec<String> = word_features
                .iter()
                .map(|id| tokenizer.stringify_feature(*id))
                .collect();
            word_feature_strings.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
            println!("{:?}", word_feature_strings);
        }
    } else {
        let vocab = Fst::new(vocab_data).unwrap();
        let mut vocab_stream = vocab.stream();
        while let Some((key, _out)) = vocab_stream.next() {
            if let Ok(s) = String::from_utf8(key.to_vec()) {
                println!("Key: {}", s);
            } else {
                println!("Error");
            }
        }
    }
}
