use std::string::FromUtf8Error;

use super::ChangeId;
use std::str::FromStr;

pub fn parse_change_id_from_bytes(bytes: &[u8]) -> Result<ChangeId, FromUtf8Error> {
    String::from_utf8(bytes.split(|b| b.eq(&b'"')).nth(3).unwrap().to_vec())
        .map(|s| ChangeId::from_str(&s).expect("Valid ChangeId"))
}

#[cfg(test)]
mod test {

    use crate::common::{parse::parse_change_id_from_bytes, ChangeId};

    #[test]
    fn test_parse_change_id_from_bytes() {
        let input = b"{\"next_change_id\": \"1882903321-1878868410-1818903289-2014357625-1957236232\", \"stashes\": []}";
        let result = parse_change_id_from_bytes(input);
        assert_eq!(
            result,
            Ok(ChangeId {
                inner: "1882903321-1878868410-1818903289-2014357625-1957236232".into()
            })
        );
    }
}
