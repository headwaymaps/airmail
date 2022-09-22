use std::{collections::HashSet, u8};

use crate::{
    model::{Model, PackedModel},
    tagger::Attribute,
    tokenizer::Tokenizer,
};
pub struct Parser {
    tokenizer: Tokenizer,
    model: Model,
}

impl<'a> Parser {
    pub fn new(packed_model_data: &[u8]) -> Parser {
        let packed_model: PackedModel = bincode2::deserialize(packed_model_data).unwrap();
        let model = Model::from(packed_model);
        let tokenizer = Tokenizer::new(&model.get_vocab());
        Parser { tokenizer, model }
    }

    pub fn parse(&self, query: &str) -> Vec<Vec<String>> {
        let mut tagger = self.model.tagger().unwrap();
        let features = self.tokenizer.tokenize(query);
        let attributes: Vec<Vec<Attribute>> = features
            .iter()
            .map(|token_attribs| {
                let attrib_vec: Vec<Attribute> = token_attribs
                    .iter()
                    .map(|id| Attribute::new(self.tokenizer.stringify_feature(*id), *id as f64))
                    .collect();
                attrib_vec
            })
            .collect();
        let mut tags_list = Vec::new();
        let mut tags_set = HashSet::new();
        {
            let tags = tagger.tag(&attributes, 0.0).unwrap();
            let tag_strs: Vec<String> = tags.0.iter().map(|tag| tag.to_string()).collect();
            tags_list.push(tag_strs.clone());
            tags_set.insert(tag_strs);
        }
        for i in 1..20 {
            for _ in 0..10 {
                let tags = tagger.tag(&attributes, i as f64 / 100.0).unwrap();
                let tag_strs: Vec<String> = tags.0.iter().map(|tag| tag.to_string()).collect();
                if !tags_set.contains(&tag_strs) {
                    tags_list.push(tag_strs.clone());
                    tags_set.insert(tag_strs);
                }
            }
        }
        tags_list
    }
}
