use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::panic;

pub enum TokenKind {
    Keyword,
    Identifier,
    Int,
    Real,
    Char,
    String,
    Operator,
    Delimiter,
    Whitespace,
    Comment,
}

// https://users.rust-lang.org/t/how-to-print-enum-values/56663/3
impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword => write!(f, "Keyword"),
            Self::Identifier => write!(f, "Identifier"),
            Self::Int => write!(f, "Integer"),
            Self::Real => write!(f, "Real"),
            Self::Char => write!(f, "Character"),
            Self::String => write!(f, "String"),
            Self::Operator => write!(f, "Operator"),
            Self::Delimiter => write!(f, "Delimiter"),
            Self::Whitespace => write!(f, "Whitespace"),
            Self::Comment => write!(f, "Comment"),
        }
    }
}

type Token = (TokenKind, String);

static KEYWORDS: [&str; 32] = [
    "and",
    "array",
    "begin",
    "boolean",
    "char",
    "dispose",
    "div",
    "do",
    "else",
    "end",
    "false",
    "forward",
    "function",
    "goto",
    "if",
    "integer",
    "label",
    "mod",
    "new",
    "nil",
    "not",
    "of",
    "or",
    "procedure",
    "program",
    "real",
    "result",
    "return",
    "then",
    "true",
    "var",
    "while",
];

static OPERATORS: [&str; 12] = [
    "=", ">", "<", "<>", ">=", "<=", "+", "-", "*", "/", "^", "@",
];

static DELIMITERS: [&str; 9] = [":=", ";", ".", "(", ")", ":", ",", "[", "]"];

fn is_digit(c: char) -> bool {
    return '0' <= c && c <= '9';
}

fn is_letter(c: char) -> bool {
    return ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

fn is_whitespace(c: char) -> bool {
    return c == ' ' || c == '\t' || c == '\n';
}

fn match_keyword(s: &str) -> &str {
    for &keyword in KEYWORDS.iter() {
        if s.starts_with(keyword) {
            return keyword;
        }
    }
    return "";
}

fn match_identifier(s: &str) -> &str {
    let mut it = s.char_indices();
    // First char must be a letter
    match it.next() {
        Some((_, c)) => {
            if !is_letter(c) {
                return "";
            }
        }
        None => {
            return &s;
        }
    }
    // Rest of chars are either letters, digits or underscores '_'
    loop {
        match it.next() {
            Some((i, c)) => {
                if !is_letter(c) && !is_digit(c) && c != '_' {
                    return &s[0..i];
                } else {
                    continue;
                }
            }
            None => {
                return &s;
            }
        }
    }
}

fn match_uint(s: &str) -> &str {
    let mut it = s.char_indices();
    loop {
        match it.next() {
            Some((i, c)) => {
                if !is_digit(c) {
                    return &s[0..i];
                }
            }
            // It will never reach here cause the EOF will match
            None => {
                return &s;
            }
        }
    }
}

fn match_real(s: &str) -> &str {
    // Decimal part
    let dec = match_uint(s);
    let mut i = dec.len();
    if i == 0 {
        return "";
    }
    // Check for the . separating decimal and fractional parts
    let mut rest_string = &s[i..];
    match rest_string.chars().next() {
        Some(c) => {
            if c != '.' {
                return "";
            }
        }
        None => {
            return "";
        }
    }
    i += 1;
    // Fractional part
    let frac = match_uint(&s[i..]);
    if frac.len() == 0 {
        return "";
    }
    i += frac.len();
    rest_string = &s[i..];
    // Optional exponential part
    let mut it = rest_string.chars();
    match it.next() {
        Some(c) => {
            // exponential part is optional so if no match then we return the previous
            if c != 'e' && c != 'E' {
                return &s[0..i];
            }
        }
        None => {
            return &s[0..i];
        }
    }
    // Suppose I match with 'e'
    i += 1;
    let mut digit = 0;
    match it.next() {
        Some(c) => {
            if !is_digit(c) && c != '+' && c != '-' {
                return &s[0..i - 1]; // maybe error?
            } else if is_digit(c) {
                digit = 1;
            }
        }
        None => {
            return &s[0..i - 1]; // maybe error? - 123.45e -> (Real, "123.45") (Id, "e") so the parser will catch it
        }
    }
    i += 1;
    rest_string = match_uint(&s[i..]);
    if rest_string.len() + digit == 0 {
        return &s[0..i - 2]; // maybe error?
    } else {
        i += rest_string.len();
        return &s[0..i];
    }
}

fn match_char(s: &str) -> &str {
    if !s.starts_with('\'') {
        return "";
    }
    let mut it = s[1..].char_indices();
    match it.next() {
        Some((_, c)) => {
            if c == '\\' {
                match it.next() {
                    Some((_, c)) => match c {
                        'n' | 't' | 'r' | '0' | '\\' | '\'' | '\"' => match it.next() {
                            Some((j, c)) => match c {
                                '\'' => &s[0..j + 2],
                                _ => panic!("Lexer error: Unterminated character literal"),
                            },
                            None => panic!("Lexer error: Unterminated character literal"),
                        },
                        _ => panic!("Lexer error: Invalid escape sequence"),
                    },
                    None => panic!("Lexer error: Unterminated character literal"),
                }
            } else if c == '\'' {
                panic!("Lexer error: empty character literal");
            } else {
                match it.next() {
                    Some((j, c)) => match c {
                        '\'' => &s[0..j + 2],
                        _ => panic!("Lexer error: Unterminated character literal"),
                    },
                    None => panic!("Lexer error: Unterminated character literal"),
                }
            }
        }
        None => panic!("Lexer error: Unterminated character literal"),
    }
}

fn match_string(s: &str) -> &str {
    if !s.starts_with('\"') {
        return "";
    }
    let mut it = s[1..].char_indices();
    let mut escaped = false;
    loop {
        match it.next() {
            Some((i, c)) => {
                if !escaped {
                    match c {
                        '\\' => escaped = true,
                        '\"' => return &s[0..i + 2],
                        _ => {}
                    }
                } else {
                    match c {
                        'n' => {}
                        't' => {}
                        'r' => {}
                        '\\' => {}
                        '\"' => {}
                        _ => {}
                    }
                    escaped = false;
                }
            }
            None => {
                panic!("Unterminated character literal");
            }
        }
    }
}

fn match_operator(s: &str) -> &str {
    for &operator in OPERATORS.iter() {
        if s.starts_with(operator) {
            return operator;
        }
    }
    return "";
}

fn match_delimiter(s: &str) -> &str {
    for &delimiter in DELIMITERS.iter() {
        if s.starts_with(delimiter) {
            return delimiter;
        }
    }
    return "";
}

fn match_whitespace(s: &str) -> &str {
    let mut it = s.char_indices();
    loop {
        match it.next() {
            Some((ind, c)) => {
                if !is_whitespace(c) {
                    return &s[0..ind];
                }
            }
            None => {
                return &s[0..0];
            }
        }
    }
}

// TODO: MISSING SOME MATCHES

/// Returns the token found at the beginning of the slice
fn next_token(src: &str) -> Token {
    // Order matters here!!!
    // It defines the priority level
    let mut cur_token = (TokenKind::Identifier, match_identifier(src));

    let keyword_lexeme = match_keyword(src);
    if keyword_lexeme.len() > cur_token.1.len() {
        cur_token = (TokenKind::Keyword, keyword_lexeme);
    }

    let uint_lexeme = match_uint(src);
    if uint_lexeme.len() > cur_token.1.len() {
        cur_token = (TokenKind::Int, uint_lexeme);
    }

    let operator_lexeme = match_operator(src);
    if operator_lexeme.len() > cur_token.1.len() {
        cur_token = (TokenKind::Operator, operator_lexeme);
    }

    let delimiter_lexeme = match_delimiter(src);
    if delimiter_lexeme.len() > cur_token.1.len() {
        cur_token = (TokenKind::Delimiter, delimiter_lexeme);
    }

    let whitespace_lexeme = match_whitespace(src);
    if whitespace_lexeme.len() > cur_token.1.len() {
        cur_token = (TokenKind::Whitespace, whitespace_lexeme);
    }

    if cur_token.1.len() == 0 {
        panic!("Lexer error");
    }

    return (cur_token.0, cur_token.1.to_string());
}

fn main() {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();
    let mut slice = &src[..];
    loop {
        let (token_kind, lexeme) = next_token(slice);
        slice = &slice[lexeme.len()..];
        println!("TokenId: {}, Lexeme: {}", token_kind, lexeme);
    }
}
