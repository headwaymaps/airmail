use autumn::{
    prelude::{throw, value, ParserExt},
    ParseResult, Parser, Span,
};
use deunicode::deunicode;
use fst::raw::Fst;

use super::AddressToken;

lazy_static! {
    static ref STREETS: Fst<Vec<u8>> = {
        let streets = Fst::new(include_bytes!("../../data/roads.fst").to_vec()).unwrap();
        streets
    };
}

#[derive(Clone)]
pub(crate) struct InvalidStreet(String);

pub(crate) fn street(
    source: &str,
    location: Span,
) -> ParseResult<Option<AddressToken>, InvalidStreet> {
    for end_offset in 0..(source.len() - location.start().character()) {
        let substr = deunicode(&source[location.start().character()..source.len() - end_offset])
            .to_ascii_lowercase();
        if STREETS.contains_key(&substr) {
            return substr
                .clone()
                .and_then(|_| value(Some(AddressToken::Street(substr.to_string()))))
                .parse(source, location);
        } else {
        }
    }
    return throw(None, InvalidStreet("No street".into())).parse(source, location);
}

#[cfg(test)]
mod test {
    use autumn::prelude::parse;

    use crate::grammar::{street::street, AddressToken};

    #[test]
    fn locality_test_roanoke() {
        assert_eq!(parse(street, "roanoke").is_success(), true);
    }

    #[test]
    fn locality_test_roa() {
        assert_eq!(parse(street, "roa").is_success(), false);
    }

    #[test]
    fn locality_test_empty() {
        assert_eq!(parse(street, "").is_success(), false);
    }

    #[test]
    fn locality_test_return() {
        assert_eq!(
            parse(street, "roanoke")
                .values()
                .collect::<Vec<&Option<AddressToken>>>(),
            vec![&Some(AddressToken::Street("roanoke".into()))]
        );
    }
}
