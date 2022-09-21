mod utils;

use airmail_lib::parser::Parser;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static PARSER: Lazy<Parser> =
    Lazy::new(|| Parser::new(include_bytes!("vocab.fst"), include_bytes!("model.crf")));

#[wasm_bindgen]
pub fn parse(query: &str) -> Vec<JsValue> {
    PARSER
        .parse(query)
        .iter()
        .map(|tag| JsValue::from_str(&tag.clone()))
        .collect()
}
