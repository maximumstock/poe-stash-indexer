use std::{fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

const LEAGUE: &str = "Crucible";
const LEAGUE_HC: &str = "Hardcore Crucible";

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
            LEAGUE => Ok(Self::Challenge),
            LEAGUE_HC => Ok(Self::ChallengeHardcore),
            _ => Err(format!("Unknown league {s}")),
        }
    }
}

impl<'a> League {
    pub fn to_str(&self) -> &'a str {
        match self {
            League::Challenge => LEAGUE,
            League::ChallengeHardcore => LEAGUE_HC,
        }
    }

    pub fn to_ident(&self) -> &'a str {
        match self {
            League::Challenge => "challenge",
            League::ChallengeHardcore => "challenge_hc",
        }
    }
}
