use autumn::prelude::parse;

pub(crate) mod locality;
pub(crate) mod street;

use locality::locality;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressToken {
    Locality(String),
    Street(String),
}

pub fn parse_address(query: &str) {
    parse(locality, query);
}
