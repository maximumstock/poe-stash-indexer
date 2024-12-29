use std::fmt::{Debug, Display};

const LIMIT: usize = 7;
const SHOW: usize = 3;

#[derive(Clone, Eq, PartialEq)]
/// Hide secrets from debug and display output.
/// Shows the first 3 characters and then replaces the rest with * if the secret is longer than 7 characters.
pub struct SecretString {
    inner: String,
}

impl SecretString {
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    pub fn expose(&self) -> &str {
        &self.inner
    }
}

fn format_secret(secret: &str) -> String {
    let mut s = String::new();
    s.push_str(&secret[0..SHOW]);
    for _ in SHOW..secret.len() {
        s.push('*');
    }
    s
}

impl Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.len() > LIMIT {
            f.write_str(&format_secret(&self.inner))
        } else {
            f.write_str("[redacted]")
        }
    }
}

impl Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SecretString;

    #[test]
    fn test_dbg() {
        let secret = SecretString::new("a secret that is longer than the limit".to_owned());
        let dbg = format!("{:?}", secret);
        assert_eq!(dbg, "a s***********************************");

        let secret = SecretString::new("ab".to_owned());
        let dbg = format!("{:?}", secret);
        assert_eq!(dbg, "[redacted]");
    }

    #[test]
    fn test_display() {
        let secret = SecretString::new("a secret that is longer than the limit".to_owned());
        let dbg = format!("{}", secret);
        assert_eq!(dbg, "a s***********************************");

        let secret = SecretString::new("ab".to_owned());
        let dbg = format!("{}", secret);
        assert_eq!(dbg, "[redacted]");
    }
}
