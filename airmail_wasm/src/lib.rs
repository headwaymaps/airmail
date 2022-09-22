mod utils;

use std::iter::FromIterator;

use airmail_lib::parser::Parser;
use js_sys::Array;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static PARSER: Lazy<Parser> = Lazy::new(|| Parser::new(include_bytes!("model.airmail")));

#[wasm_bindgen]
pub fn parse(query: &str) -> Vec<Array> {
    PARSER
        .parse(query)
        .iter()
        .map(|tag_sequence| {
            Array::from_iter(
                tag_sequence
                    .iter()
                    .map(|tag| JsValue::from_str(&tag.clone())),
            )
        })
        .collect()
}
