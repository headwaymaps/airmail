use std::u8;

use crate::tokenizer::Tokenizer;
use crfs::{Attribute, Model};
use fst::raw::Fst;
pub struct Parser<'a> {
    tokenizer: Tokenizer,
    model: Model<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(vocab_data: &[u8], model_data: &'a [u8]) -> Parser<'a> {
        let fst = Fst::new(vocab_data.to_vec()).unwrap();
        let tokenizer = Tokenizer::new(&fst);
        let model = Model::new(model_data).unwrap();
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
