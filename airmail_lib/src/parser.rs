use std::u8;

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

    pub fn parse(&self, query: &str) -> Vec<String> {
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

        let tags: Vec<String> = tagger
            .tag(&attributes)
            .unwrap()
            .iter()
            .map(|tag| tag.to_string())
            .collect();
        tags
    }
}
