use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum League {
    Challenge,
    ChallengeHardcore,
}

impl FromStr for League {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sentinel" => Ok(Self::Challenge),
            "Hardcore Sentinel" => Ok(Self::ChallengeHardcore),
            _ => Err(format!("Unknown league {}", s)),
        }
    }
}

impl<'a> League {
    pub fn to_str(&self) -> &'a str {
        match self {
            League::Challenge => "Sentinel",
            League::ChallengeHardcore => "Hardcore Sentinel",
        }
    }

    pub fn to_ident(&self) -> &'a str {
        match self {
            League::Challenge => "challenge",
            League::ChallengeHardcore => "challenge_hc",
        }
    }
}
