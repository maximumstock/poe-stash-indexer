use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub const CHALLENGE_LEAGUE: &str = "Settlers";
pub const CHALLENGE_LEAGUE_HC: &str = "Hardcore Settlers";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A newtype struct that represents a League in the Path of Exile API data.
/// Usually just handling the string
pub struct League {
    inner: String,
}

impl League {
    pub fn new(league: String) -> Self {
        Self { inner: league }
    }

    pub fn is_hc(&self) -> bool {
        self.inner.contains("HC") || self.inner.contains("Hardcore")
    }
}

impl AsRef<str> for League {
    fn as_ref(&self) -> &str {
        self.inner.as_str()
    }
}
