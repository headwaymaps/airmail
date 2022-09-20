use std::{fs::File, io::Read};

use airmail::tokenizer::Tokenizer;
use clap::Parser;
use crfsuite::{Attribute, Model};
use fst::raw::Fst;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The vocabulary file to use.
    #[clap(long, value_parser)]
    vocab: String,
    /// The model file to use.
    #[clap(long, value_parser)]
    model: String,
    /// The address string to parse.
    #[clap(long, value_parser)]
    str: String,
}

fn main() {
    let args = Args::parse();

    let mut vocab_data = vec![];
    File::open(args.vocab)
        .unwrap()
        .read_to_end(&mut vocab_data)
        .unwrap();

    let fst = Fst::new(vocab_data).unwrap();
    let tokenizer = Tokenizer::new(&fst);
    let features = tokenizer.tokenize(&args.str);
    for word_features in &features {
        let mut word_feature_strings: Vec<String> = word_features
            .iter()
            .map(|id| tokenizer.stringify_feature(*id))
            .collect();
        word_feature_strings.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
        println!("{:?}", word_feature_strings);
    }

    let model = Model::from_file(&args.model).unwrap();
    let mut tagger = model.tagger().unwrap();

    let attributes: Vec<Vec<Attribute>> = features
        .iter()
        .map(|token_attribs| {
            let attrib_vec: Vec<Attribute> = token_attribs
                .iter()
                .map(|id| Attribute::new(tokenizer.stringify_feature(*id), *id as f64))
                .collect();
            attrib_vec
        })
        .collect();

    let tagged = tagger.tag(&attributes).unwrap();
    println!("Tagged: {:?}", tagged);
}
