#[derive(Debug)]
pub(crate) struct ChangeID<'a> {
    inner: &'a str,
}

impl<'a> ChangeID<'a> {
    pub fn from_str(input: &'a str) -> Result<Self, Box<dyn std::error::Error>> {
        let is_valid = input
            .split("-")
            .map(|x| x.parse::<u32>())
            .all(|x| x.is_ok());

        match is_valid {
            true => Ok(Self { inner: input }),
            false => Err("derp".into()),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_from_str_success() {
        let change_id =
            super::ChangeID::from_str("850662131-863318628-825558626-931433265-890834941");

        assert!(change_id.is_ok(),);
        assert_eq!(
            change_id.unwrap().inner,
            "850662131-863318628-825558626-931433265-890834941"
        );
    }

    #[test]
    fn test_from_str_err() {
        assert!(
            super::ChangeID::from_str("850662A31-863318628-825558626-931433265-890834941").is_err(),
        );
    }
}
