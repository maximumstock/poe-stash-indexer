use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct ChangeId {
    pub(crate) inner: String,
}

impl Display for ChangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl std::str::FromStr for ChangeId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_valid = s.split('-').map(|x| x.parse::<u32>()).all(|x| x.is_ok());

        match is_valid {
            true => Ok(Self {
                inner: s.to_owned(),
            }),
            false => Err("derp".into()),
        }
    }
}

impl From<ChangeId> for String {
    fn from(change_id: ChangeId) -> Self {
        change_id.inner 
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str_success() {
        let change_id = ChangeId::from_str("850662131-863318628-825558626-931433265-890834941");

        assert!(change_id.is_ok(),);
        assert_eq!(
            change_id.unwrap().inner,
            "850662131-863318628-825558626-931433265-890834941"
        );
    }

    #[test]
    fn test_from_str_err() {
        assert!(
            super::ChangeId::from_str("850662A31-863318628-825558626-931433265-890834941").is_err(),
        );
    }
}
