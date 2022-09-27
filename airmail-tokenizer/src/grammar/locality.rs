use autumn::{
    prelude::{throw, value, ParserExt},
    ParseResult, Parser, Span,
};
use deunicode::deunicode;
use fst::raw::Fst;

use super::AddressToken;

lazy_static! {
    static ref LOCALITIES: Fst<Vec<u8>> = {
        let localities = Fst::new(include_bytes!("../../data/localities.fst").to_vec()).unwrap();
        localities
    };
}

#[derive(Clone)]
pub(crate) struct InvalidLocality(String);

pub(crate) fn locality(
    source: &str,
    location: Span,
) -> ParseResult<Option<AddressToken>, InvalidLocality> {
    for end_offset in 0..(source.len() - location.start().character()) {
        let substr = deunicode(&source[location.start().character()..source.len() - end_offset])
            .to_ascii_lowercase();
        if LOCALITIES.contains_key(&substr) {
            return substr
                .clone()
                .and_then(|_| value(Some(AddressToken::Locality(substr.to_string()))))
                .parse(source, location);
        } else {
        }
    }
    return throw(None, InvalidLocality("No locality".into())).parse(source, location);
}

#[cfg(test)]
mod test {
    use autumn::prelude::parse;

    use crate::grammar::{locality::locality, AddressToken};

    #[test]
    fn locality_test_seattle() {
        assert_eq!(parse(locality, "seattle").is_success(), true);
    }

    #[test]
    fn locality_test_sea() {
        assert_eq!(parse(locality, "sea").is_success(), false);
    }

    #[test]
    fn locality_test_empty() {
        assert_eq!(parse(locality, "").is_success(), false);
    }

    #[test]
    fn locality_test_return() {
        assert_eq!(
            parse(locality, "seattle")
                .values()
                .collect::<Vec<&Option<AddressToken>>>(),
            vec![&Some(AddressToken::Locality("seattle".into()))]
        );
    }
}
