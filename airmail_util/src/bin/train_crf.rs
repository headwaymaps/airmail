use std::{
    fs::File,
    io::Read,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    time::Duration,
};

use airmail_lib::{
    lp_file_stream::{LpEntryToken, LpFileStream},
    tokenizer::Tokenizer,
};
use clap::Parser;
use crfsuite::{Algorithm, Attribute, GraphicalModel, Trainer};
use fst::raw::Fst;
use rand::{thread_rng, Rng};
use rayon::prelude::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The vocabulary file to use.
    #[clap(long, value_parser)]
    vocab: String,
    /// The tsv training file to use.
    #[clap(long, value_parser)]
    tsv: String,
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
    let fst = Fst::new(vocab_data).unwrap();
    let tokenizer = Tokenizer::new(&fst);

    let tsv_stream = LpFileStream::new(args.tsv).unwrap();

    rayon::scope(|scope| {
        let (sender, reciever): (
            SyncSender<(Box<Vec<Vec<(String, f64)>>>, Vec<String>)>,
            Receiver<(Box<Vec<Vec<(String, f64)>>>, Vec<String>)>,
        ) = sync_channel(1000000);
        scope.spawn(move |_| {
            let mut trainer = Trainer::new(true);
            trainer
                .select(Algorithm::PA, GraphicalModel::CRF1D)
                .unwrap();
            let mut counter = 0usize;
            while let Ok((attribute_vec_per_token, target_per_token)) =
                reciever.recv_timeout(Duration::from_secs(1))
            {
                let group = if counter % 100 == 0 { 1 } else { 0 };
                let actual_attributes: Vec<Vec<Attribute>> = attribute_vec_per_token
                    .iter()
                    .map(|token_attribs| {
                        let attrib_vec: Vec<Attribute> = token_attribs
                            .iter()
                            .map(|(name, id)| Attribute::new(name, *id))
                            .collect();
                        attrib_vec
                    })
                    .collect();
                trainer
                    .append(&actual_attributes, &target_per_token, group)
                    .unwrap();
                counter += 1;
                if counter % 100000 == 0 {
                    println!("Processed {} lines", counter);
                }
            }
            println!("training");
            trainer.train("model.crf", 1).unwrap();
            println!("done training");
            panic!();
        });
        tsv_stream.take(200000).par_bridge().for_each(|tsv_item| {
            let mut attribute_vec_per_token = vec![];
            let mut target_per_token = vec![];
            let all_tokens: Vec<&LpEntryToken> = tsv_item
                .tokens
                .iter()
                .filter(|token| token.label != "FSEP")
                .collect();
            let tokens_len = all_tokens.len();
            let tokens_to_use = if tokens_len < 2 || thread_rng().gen::<bool>() {
                all_tokens
            } else {
                all_tokens
                    .into_iter()
                    .take(thread_rng().gen_range(1..tokens_len))
                    .collect()
            };
            for token in tokens_to_use {
                if token.label == "FSEP" {
                    continue;
                }
                let actual_label = match token.label.as_str() {
                    // The nuance associated with these different labels is too much for our parser to deal with given the size budget.
                    "level" => "unit",
                    "entrance" => "unit",
                    "staircase" => "unit",
                    // This is regrettable but this is a tiny parser and it doesn't have room to memorize what every `house` token looks like.
                    "house" => continue,
                    // Postal codes aren't nearly as important as other aspects of geocoding, but don't completely ignore them.
                    "postcode" => {
                        if thread_rng().gen::<f64>() < 0.1 {
                            "postcode"
                        } else {
                            continue;
                        }
                    }
                    // This is something we can deal with downstream.
                    "city" => "locality",
                    "suburb" => "locality",
                    "city_district" => "locality",
                    // Similarly, a structured search system can easily deal with this ambiguity.
                    "state_district" => "region",
                    "state" => "region",
                    "island" => "region",
                    x => x,
                };
                let features = {
                    let split: Vec<&str> = token.transliterated.split_ascii_whitespace().collect();
                    let cat_token = split.join("");
                    let all_features = tokenizer.tokenize(&cat_token);
                    if all_features.len() != 1 {
                        continue;
                    }
                    all_features[0].clone()
                };
                let attributes: Vec<(String, f64)> = features
                    .iter()
                    .map(|id| {
                        let name = tokenizer.stringify_feature(*id);
                        (name.clone(), *id as f64)
                    })
                    .collect();
                attribute_vec_per_token.push(attributes);
                target_per_token.push(actual_label.to_string());
            }
            match sender
                .clone()
                .send((Box::new(attribute_vec_per_token), target_per_token))
            {
                Ok(_) => {}
                Err(_) => {
                    println!("Failed to send");
                    panic!();
                }
            }
        });
        drop(sender);
    });
}
