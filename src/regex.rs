use std::collections::HashMap;
use std::ops::{Index, Range};

/// Captured keys which can be identified by numeric index or by name.
#[derive(Hash, PartialEq)]
enum CaptureKey {
    Index(usize),
    Name(String),
}

impl Eq for CaptureKey {}

/// The result of the capturing function. Can be indexed by number (usize) or '&str' to obtain
/// the matches.
pub struct Captured {
    text: String,
    captures: HashMap<CaptureKey, Range<usize>>,
}

impl Index<usize> for Captured {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.text[self.captures[&CaptureKey::Index(index)].clone()]
    }
}

impl Index<&str> for Captured {
    type Output = str;

    fn index(&self, index: &str) -> &Self::Output {
        &self.text[self.captures[&CaptureKey::Name(index.into())].clone()]
    }
}

/// Returns the captures from the 'input' data matched by 'regex'.
/// The input is lossy translated to UTF8.
pub fn captures_utf8(input: &[u8], regex: &str) -> Captured {
    let mut captures = HashMap::new();
    use regex::Regex;
    let re = Regex::new(regex).expect("valid regex");
    let text = String::from_utf8_lossy(input).to_string();

    use CaptureKey::*;

    if let Some(c) = re.captures(&text) {
        for n in 0..c.len() {
            if let Some(m) = c.get(n) {
                captures.insert(Index(n), m.range());
            }
        }

        for n in re.capture_names() {
            if let (Some(n), Some(m)) = (n, c.name(n.unwrap_or_default())) {
                captures.insert(Name(String::from(n)), m.range());
            }
        }
    };

    Captured { text, captures }
}

/// Checks if the input (lossy translated to utf8) matches the given regex.
/// Returns a tuple of the test outcome and the utf8 string (for diagnostics).
pub fn regex_match_utf8(input: &[u8], regex: &str) -> (bool, String) {
    use regex::Regex;
    let re = Regex::new(regex).expect("compiled regex");
    let text = String::from_utf8_lossy(input);
    (re.is_match(&text), text.into_owned())
}

/// Checks if the input matches the given regex as bytes.
/// Returns a tuple of the test outcome and the input as lossy utf8 string (for diagnostics).
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
        let captures = captures_utf8(b"Hello World!", "(?P<first>[^ ]*) (?P<second>[^ ]*)");

        assert_eq!(&captures[0], "Hello World!");
        assert_eq!(&captures[1], "Hello");
        assert_eq!(&captures[2], "World!");
        assert_eq!(&captures["first"], "Hello");
        assert_eq!(&captures["second"], "World!");
    }
}
