use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    time::Duration,
    usize,
};

use airmail_lib::lp_file_stream::{LpEntryToken, LpFileStream};
use clap::{command, Arg, ArgAction};
use fst::SetBuilder;
use rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use tokenizers::{
    models::unigram::{Unigram, UnigramTrainer},
    normalizers::{Sequence, Strip, NFC},
    processors::byte_level::ByteLevel,
    TokenizerBuilder,
};

struct ClampedExpandedVocab<'a> {
    _token_frequencies: &'a HashMap<(String, String), usize>,
    label: String,
    iter: std::collections::hash_map::Iter<'a, (String, String), usize>,
    token: (String, String),
    remaining_count: usize,
}

impl<'a> ClampedExpandedVocab<'a> {
    fn new(
        token_frequencies: &'a HashMap<(String, String), usize>,
        label: &str,
    ) -> ClampedExpandedVocab<'a> {
        ClampedExpandedVocab {
            _token_frequencies: token_frequencies,
            label: label.to_string(),
            iter: token_frequencies.iter(),
            token: ("".to_string(), "".to_string()),
            remaining_count: 0,
        }
    }
}

impl<'a> Iterator for ClampedExpandedVocab<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_count == 0 {
            loop {
                if let Some((new_token, new_frequency)) = self.iter.next() {
                    if new_token.1 != self.label {
                        continue;
                    }
                    self.token = new_token.clone();
                    self.remaining_count = usize::min(*new_frequency, 10);
                    return Some(self.token.clone().0);
                } else {
                    return None;
                }
            }
        } else {
            self.remaining_count -= 1;
            Some(self.token.clone().0)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .arg(
            Arg::new("f")
                .long("files")
                .required(true)
                .takes_value(true)
                .multiple_values(true),
        )
        .arg(Arg::new("o").long("out").takes_value(true).required(true))
        .arg(Arg::new("p").long("pretty").action(ArgAction::SetTrue))
        .get_matches();

    if let Some(files) = matches.get_many::<String>("f") {
        let out_file = matches.get_one::<String>("o").unwrap();
        rayon::scope(|scope| {
            let (sender, reciever): (SyncSender<LpEntryToken>, Receiver<LpEntryToken>) =
                sync_channel(10000);
            scope.spawn(move |_| {
                let mut token_frequencies = HashMap::new();
                let mut all_labels = HashSet::new();
                while let Ok(token) = reciever.recv_timeout(Duration::from_secs(1)) {
                    all_labels.insert(token.label.clone());
                    if let Some(prev) =
                        token_frequencies.get(&(token.transliterated.clone(), token.label.clone()))
                    {
                        token_frequencies.insert(
                            (token.transliterated.clone(), token.label.clone()),
                            prev + 1,
                        );
                    } else {
                        token_frequencies
                            .insert((token.transliterated.clone(), token.label.clone()), 1);
                    }
                }
                let mut vocab: HashSet<String> = HashSet::new();
                let per_label_vocab: Vec<HashSet<String>> = all_labels
                    .par_iter()
                    .map(|label| {
                        println!("Generating vocab for label `{}`", label);
                        let data = ClampedExpandedVocab::new(&token_frequencies, &label);
                        let vocab_size: u32 = 200000;
                        let mut trainer = UnigramTrainer::builder()
                            .show_progress(true)
                            .vocab_size(vocab_size)
                            .build()
                            .unwrap();

                        let mut tokenizer = TokenizerBuilder::new()
                            .with_model(Unigram::default())
                            .with_normalizer(Some(Sequence::new(vec![
                                Strip::new(true, true).into(),
                                NFC.into(),
                            ])))
                            .with_pre_tokenizer(Some(ByteLevel::default()))
                            .with_post_processor(Some(ByteLevel::default()))
                            .with_decoder(Some(ByteLevel::default()))
                            .build()
                            .unwrap();
                        let tokenizer_impl = tokenizer.train(&mut trainer, data).unwrap();
                        let vocab_map = tokenizer_impl.get_vocab(false);
                        let vocab: HashSet<String> = vocab_map
                            .into_keys()
                            .map(|s| {
                                if let Some(key) = s.strip_prefix("Ġ") {
                                    key.to_string()
                                } else {
                                    s
                                }
                            })
                            .collect();
                        println!("{} vocab contains {} items", label, vocab.len());
                        vocab
                    })
                    .collect();

                for label_vocab in per_label_vocab {
                    vocab.extend(label_vocab);
                }

                let mut vocab: Vec<&String> = vocab.iter().collect();
                vocab.sort();
                let mut builder = SetBuilder::new(File::create(out_file).unwrap()).unwrap();

                vocab
                    .iter()
                    .for_each(|key| builder.insert(key.clone()).unwrap());
                builder.finish().unwrap();
            });
            for f in files {
                println!("Processing file: {}", f);
                let stream = LpFileStream::new(f.to_string()).unwrap();
                stream.par_bridge().for_each(|entry| {
                    for token in &entry.tokens {
                        sender.clone().send(token.clone()).unwrap();
                    }
                });
            }
        });

        Ok(())
    } else {
        Ok(())
    }
}
