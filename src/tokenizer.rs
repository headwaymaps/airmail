use std::collections::{HashMap, HashSet};

use deunicode::deunicode;
use fst::{raw::Fst, Streamer};

pub struct Tokenizer {
    feature_ids: HashMap<String, u32>,
    feature_strings: HashMap<u32, String>,
    feature_count: u32,
}

impl Tokenizer {
    pub fn new(vocab: &Fst<Vec<u8>>) -> Tokenizer {
        let mut vocab_stream = vocab.stream();
        let mut feature_ids = HashMap::new();
        let mut feature_strings = HashMap::new();
        let mut feature_id = 0u32;
        while let Some((key, _out)) = vocab_stream.next() {
            if let Ok(s) = String::from_utf8(key.to_vec()) {
                let vocab_item = if let Some(after_prefix) = s.strip_prefix("Ġ") {
                    after_prefix
                } else {
                    &s
                };
                if feature_ids.contains_key(vocab_item) {
                    continue;
                }
                feature_ids.insert(vocab_item.to_string(), feature_id);
                feature_strings.insert(feature_id, vocab_item.to_string());
                feature_id += 1;
            } else {
                println!("Error");
                panic!()
            }
        }
        Tokenizer {
            feature_ids,
            feature_strings,
            feature_count: feature_id,
        }
    }

    pub fn tokenize(&self, string: &str) -> Vec<Vec<u32>> {
        let transliterated = deunicode(string).to_ascii_lowercase();
        let words = transliterated.split_whitespace();
        let mut feature_vecs = vec![];
        for word in words {
            let mut feature_set = HashSet::new();
            self.features_for_ascii_word(word, &mut feature_set);
            let features: Vec<u32> = feature_set.into_iter().collect();
            feature_vecs.push(features);
        }
        feature_vecs
    }

    fn features_for_ascii_word_recursive(&self, word: &str, seed_set: &mut HashSet<u32>) {
        for i in 1..=word.len() {
            let prefix = &word[0..i];
            if let Some(feature_id) = self.feature_ids.get(prefix) {
                seed_set.insert(*feature_id);
                self.features_for_ascii_word_recursive(&word[i..word.len()], seed_set);
            }
        }
    }

    fn features_for_ascii_word(&self, word: &str, seed_set: &mut HashSet<u32>) {
        self.features_for_ascii_word_recursive(word, seed_set);
        if !word.is_empty()
            && word.chars().all(|ch| match ch {
                '0' => true,
                '1' => true,
                '2' => true,
                '3' => true,
                '4' => true,
                '5' => true,
                '6' => true,
                '7' => true,
                '8' => true,
                '9' => true,
                '-' => true,
                _ => false,
            })
        {
            let digit_count = word
                .chars()
                .filter(|ch| match ch {
                    '0' => true,
                    '1' => true,
                    '2' => true,
                    '3' => true,
                    '4' => true,
                    '5' => true,
                    '6' => true,
                    '7' => true,
                    '8' => true,
                    '9' => true,
                    _ => false,
                })
                .count();
            if digit_count > 0 {
                seed_set.insert(self.feature_count + digit_count as u32);
            }
        }
    }

    pub fn stringify_feature(&self, feature: u32) -> String {
        if let Some(s) = self.feature_strings.get(&feature) {
            s.clone()
        } else {
            format!("D:{}", feature - self.feature_count)
        }
    }
}
