const PRICE_PATTERN: &str = "^(~b/o|~price) ([0-9]+[\\./]?[0-9]*) ([a-zA-Z-]*)";

#[derive(Debug)]
pub struct Price<'a> {
    pub ratio: f32,
    pub item: &'a str,
}

pub struct PriceParser {
    regex: regex::Regex,
}

impl Default for PriceParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceParser {
    pub fn new() -> Self {
        Self {
            regex: regex::Regex::new(PRICE_PATTERN)
                .expect("Failed to compile regex from PRICE_PATTERN"),
        }
    }

    pub fn parse_price<'a>(&self, note: &'a str) -> Option<Price<'a>> {
        if let Some(groups) = self.regex.captures(note) {
            let ratio = groups.get(2).map(|x| x.as_str());
            let item = groups.get(3).map(|x| x.as_str());

            if let (Some(ratio), Some(item)) = (ratio, item) {
                if let Ok(parsed_ration) = Self::extract_ratio(ratio) {
                    return Some(Price {
                        ratio: parsed_ration,
                        item,
                    });
                }
            }
        }
        None
    }

    fn extract_ratio(input: &str) -> Result<f32, ()> {
        if input.contains('/') {
            let mut parts = input.split('/').take(2);
            let numerator = parts.next().and_then(|x| x.parse::<f32>().ok());
            let denominator = parts.next().and_then(|x| x.parse::<f32>().ok());

            if numerator == Some(0.0) || denominator == Some(0.0) {
                return Err(());
            }

            if let (Some(left), Some(right)) = (numerator, denominator) {
                return Ok(left / right);
            }

            Err(())
        } else {
            input.parse::<f32>().map_err(|_| ())
        }
    }

    #[allow(dead_code)]
    pub fn is_price(&self, note: &str) -> bool {
        self.regex.is_match(note)
    }
}

#[cfg(test)]
mod tests {
    use crate::note_parser::PriceParser;

    use super::Price;

    const VALID: [(&str, Price); 8] = [
        (
            "~b/o 050 chaos",
            Price {
                item: "chaos",
                ratio: 50f32,
            },
        ),
        (
            "~b/o 100 chaos",
            Price {
                item: "chaos",
                ratio: 100f32,
            },
        ),
        (
            "~b/o 12/19 chaos",
            Price {
                item: "chaos",
                ratio: 12f32 / 19f32,
            },
        ),
        (
            "~b/o 1.2 exalted",
            Price {
                item: "exalted",
                ratio: 1.2f32,
            },
        ),
        (
            "~b/o 6 wisdom 160minion dmg + 80 minion Life",
            Price {
                item: "wisdom",
                ratio: 6f32,
            },
        ),
        (
            "~price 10/5000 chaos",
            Price {
                item: "chaos",
                ratio: 10f32 / 5000f32,
            },
        ),
        (
            "~price 1/5 forge-of-the-phoenix-map",
            Price {
                item: "forge-of-the-phoenix-map",
                ratio: 0.2,
            },
        ),
        (
            "~b/o 01.323 exalted",
            Price {
                item: "exalted",
                ratio: 1.323,
            },
        ),
    ];

    const SYNTACTICALLY_INVALID: [&str; 8] = [
        "",
        "~price  chaos",
        "~b/o  chaos",
        ">10",
        "Legacy",
        "~b/o 13..123123.123 chaos",
        "~b/o 13//123123 chaos",
        "~b/o 13/123/1 chaos",
    ];

    const SEMANTICALLY_INVALID: [&str; 2] = ["~price 1/0 chaos", "~price 0/1 chaos"];

    #[test]
    fn test_price_parsing() {
        let parser = PriceParser::new();

        for (input, _expected) in VALID {
            assert!(parser.is_price(input));
            assert!(matches!(parser.parse_price(input), Some(_expected)));
        }

        for input in SYNTACTICALLY_INVALID {
            assert!(!parser.is_price(input));
            assert!(parser.parse_price(input).is_none());
        }

        for input in SEMANTICALLY_INVALID {
            assert!(parser.is_price(input));
            assert!(parser.parse_price(input).is_none());
        }
    }
}
