use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Text(String),
    Object(Vec<(String, Vdf)>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Object(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Text(_) => None,
        }
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Vdf::Text(value)) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn entries(&self) -> &[(String, Vdf)] {
        match self {
            Vdf::Object(entries) => entries,
            Vdf::Text(_) => &[],
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Text(String),
    Open,
    Close,
}

pub fn parse(input: &str) -> Option<Vdf> {
    let tokens = tokenize(input);
    let mut iter = tokens.into_iter().peekable();
    let root = parse_entries(&mut iter)?;
    Some(Vdf::Object(root))
}

fn parse_entries(iter: &mut Peekable<std::vec::IntoIter<Token>>) -> Option<Vec<(String, Vdf)>> {
    let mut entries = Vec::new();
    loop {
        match iter.next() {
            None | Some(Token::Close) => return Some(entries),
            Some(Token::Open) => return None,
            Some(Token::Text(key)) => {
                let value = match iter.next()? {
                    Token::Text(value) => Vdf::Text(value),
                    Token::Open => Vdf::Object(parse_entries(iter)?),
                    Token::Close => return None,
                };
                entries.push((key, value));
            }
        }
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '{' => {
                chars.next();
                tokens.push(Token::Open);
            }
            '}' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '"' => {
                chars.next();
                tokens.push(Token::Text(read_quoted(&mut chars)));
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    for skipped in chars.by_ref() {
                        if skipped == '\n' {
                            break;
                        }
                    }
                }
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => tokens.push(Token::Text(read_bare(&mut chars))),
        }
    }
    tokens
}

fn read_quoted(chars: &mut Peekable<Chars>) -> String {
    let mut value = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => value.push('\n'),
                Some('t') => value.push('\t'),
                Some(other) => value.push(other),
                None => break,
            },
            other => value.push(other),
        }
    }
    value
}

fn read_bare(chars: &mut Peekable<Chars>) -> String {
    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
            break;
        }
        value.push(c);
        chars.next();
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_objects_and_escapes() {
        let doc = parse(
            r#"
            // comment
            "libraryfolders"
            {
                "0"
                {
                    "path"      "C:\\Program Files (x86)\\Steam"
                    "apps" { "730" "123" }
                }
            }
            "#,
        )
        .unwrap();
        let folder = doc.get("libraryfolders").unwrap().get("0").unwrap();
        assert_eq!(folder.text("path"), Some("C:\\Program Files (x86)\\Steam"));
        assert_eq!(folder.get("apps").unwrap().text("730"), Some("123"));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let doc = parse(r#""AppState" { "LastPlayed" "42" }"#).unwrap();
        assert_eq!(doc.get("appstate").unwrap().text("lastplayed"), Some("42"));
    }

    #[test]
    fn rejects_malformed_input() {
        assert!(parse(r#""a" { "b" "#).is_none());
        assert!(parse(r#"{ "a" "b" }"#).is_none());
    }
}
