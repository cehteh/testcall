use std::collections::HashMap;

/// Captured keys which can be identified by numeric index or by name.
#[derive(Hash, PartialEq)]
pub enum CaptureKey {
    Index(usize),
    Name(String),
}

impl Eq for CaptureKey {}

pub fn captures_utf8(input: &[u8], regex: &str) -> HashMap<CaptureKey, String> {
    let mut captures = HashMap::new();
    use regex::Regex;
    let re = Regex::new(regex).expect("valid regex");
    let text = String::from_utf8_lossy(input);

    use CaptureKey::*;

    if let Some(c) = re.captures(&text) {
        for n in 0..c.len() {
            if let Some(m) = c.get(n) {
                captures.insert(Index(n), String::from(m.as_str()));
            }
        }

        for n in re.capture_names() {
            if let (Some(n), Some(m)) = (n, c.name(n.unwrap_or_default())) {
                captures.insert(Name(String::from(n)), String::from(m.as_str()));
            }
        }
    };

    captures
}

pub fn regex_match_utf8(input: &[u8], regex: &str) -> (bool, String) {
    use regex::Regex;
    let re = Regex::new(regex).expect("compiled regex");
    let text = String::from_utf8_lossy(input);
    (re.is_match(&text), text.into_owned())
}

pub fn regex_match_bytes(input: &[u8], regex: &str) -> (bool, String) {
    use regex::bytes::Regex;
    let re = Regex::new(regex).expect("compiled regex");
    (
        re.is_match(input),
        String::from_utf8_lossy(input).into_owned(),
    )
}

#[cfg(test)]
#[cfg(unix)]
mod test {
    use super::*;

    #[test]
    fn captures() {
        use CaptureKey::*;

        let captures = captures_utf8(b"Hello World!", "(?P<first>[^ ]*) (?P<second>[^ ]*)");

        assert_eq!(captures[&Index(0)], "Hello World!");
        assert_eq!(captures[&Index(1)], "Hello");
        assert_eq!(captures[&Index(2)], "World!");
        assert_eq!(captures[&Name(String::from("first"))], "Hello");
        assert_eq!(captures[&Name(String::from("second"))], "World!");
    }

}
