use std::{fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum League {
    Challenge,
    ChallengeHardcore,
}

impl Debug for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl FromStr for League {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Kalandra" => Ok(Self::Challenge),
            "Hardcore Kalandra" => Ok(Self::ChallengeHardcore),
            _ => Err(format!("Unknown league {}", s)),
        }
    }
}

impl<'a> League {
    pub fn to_str(&self) -> &'a str {
        match self {
            League::Challenge => "Kalandra",
            League::ChallengeHardcore => "Hardcore Kalandra",
        }
    }

    pub fn to_ident(&self) -> &'a str {
        match self {
            League::Challenge => "challenge",
            League::ChallengeHardcore => "challenge_hc",
        }
    }
}
