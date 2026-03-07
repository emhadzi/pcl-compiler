use std::panic;

#[derive(Debug, Copy, Clone, PartialEq)]
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
    EOF,
    Error,
}

// https://users.rust-lang.org/t/how-to-print-enum-values/56663/3
// impl Display for TokenKind {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Keyword => write!(f, "Keyword"),
//             Self::Identifier => write!(f, "Identifier"),
//             Self::Int => write!(f, "Integer"),
//             Self::Real => write!(f, "Real"),
//             Self::Char => write!(f, "Character"),
//             Self::String => write!(f, "String"),
//             Self::Operator => write!(f, "Operator"),
//             Self::Delimiter => write!(f, "Delimiter"),
//             Self::Whitespace => write!(f, "Whitespace"),
//             Self::Comment => write!(f, "Comment"),
//             Self::EOF => write!(f, "EOF"),
//             Self::Error => write!(f, "Error"),
//         }
//     }
// }

pub type Token = (TokenKind, String);

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

pub struct Lexer<'a> {
    slice: &'a str,
    pub line: usize,
    pub column: usize,
}

fn is_digit(c: char) -> bool {
    return '0' <= c && c <= '9';
}

fn is_letter(c: char) -> bool {
    return ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z');
}

fn is_whitespace(c: char) -> bool {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            slice: src,
            line: 1,
            column: 1,
        }
    }

    fn match_keyword(s: &str) -> Option<&str> {
        for &keyword in KEYWORDS.iter() {
            if s.starts_with(keyword) {
                return Some(keyword);
            }
        }
        return None;
    }

    fn match_identifier(s: &str) -> Option<&str> {
        let mut it = s.char_indices();
        // First char must be a letter
        match it.next() {
            Some((_, c)) => {
                if !is_letter(c) {
                    return None;
                }
            }
            None => {
                return None;
            }
        }
        // Rest of chars are either letters, digits or underscores '_'
        loop {
            match it.next() {
                Some((i, c)) => {
                    if !is_letter(c) && !is_digit(c) && c != '_' {
                        return Some(&s[0..i]);
                    }
                }
                // EOF
                None => {
                    return Some(s);
                }
            }
        }
    }

    fn match_uint(s: &str) -> Option<&str> {
        let mut it = s.char_indices();
        loop {
            match it.next() {
                Some((0, c)) => {
                    if !is_digit(c) {
                        return None;
                    }
                }
                Some((i, c)) => {
                    if !is_digit(c) {
                        return Some(&s[0..i]);
                    }
                }
                // EOF
                None => {
                    return Some(s);
                }
            }
        }
    }

    fn match_real(s: &str) -> Option<&str> {
        // Decimal part
        let dec: &str;
        match Self::match_uint(s) {
            Some(d) => {
                dec = d;
            }
            None => {
                return None;
            }
        }
        let mut i = dec.len();
        // Check for the . separating decimal and fractional parts
        let mut rest_string = &s[i..];
        match rest_string.chars().next() {
            Some(c) => {
                if c != '.' {
                    return None;
                }
            }
            None => {
                return None;
            }
        }
        i += 1;
        // Fractional part
        let frac: &str;
        match Self::match_uint(&s[i..]) {
            Some(f) => {
                frac = f;
            }
            None => {
                return None;
            }
        }
        i += frac.len();
        rest_string = &s[i..];
        // Optional exponential part
        let mut it = rest_string.chars();
        match it.next() {
            Some(c) => {
                // exponential part is optional so if no match then we return the previous
                if c != 'e' && c != 'E' {
                    return Some(&s[0..i]);
                }
            }
            None => {
                return Some(&s[0..i]);
            }
        }
        // Suppose I match with 'e'
        i += 1;
        let mut digit = 0;
        match it.next() {
            Some(c) => {
                if !is_digit(c) && c != '+' && c != '-' {
                    return Some(&s[0..i - 1]);
                } else if is_digit(c) {
                    digit = 1;
                }
            }
            None => {
                return Some(&s[0..i - 1]); // maybe error? - 123.45e -> (Real, "123.45") (Id, "e") so the parser will catch it
            }
        }
        i += 1;
        match Self::match_uint(&s[i..]) {
            Some(exp_part) => {
                i += exp_part.len();
                return Some(&s[0..i]);
            }
            None => {
                if digit == 0 {
                    return Some(&s[0..i - 2]); // maybe error?
                } else {
                    return Some(&s[0..i]);
                }
            }
        }
    }

    // TODO: Make it return Option
    fn match_char(s: &str) -> Option<&str> {
        if !s.starts_with('\'') {
            return None;
        }
        // No other token starts with ' so we can panic
        let mut it = s[1..].char_indices();
        match it.next() {
            Some((_, c)) => {
                if c == '\\' {
                    match it.next() {
                        Some((_, c)) => match c {
                            'n' | 't' | 'r' | '0' | '\\' | '\'' | '\"' => match it.next() {
                                Some((j, c)) => match c {
                                    '\'' => Some(&s[0..j + 2]),
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
                } else if c == '\"' {
                    panic!(
                        "Lexer error: character literal cannot contain an unescaped double quote"
                    );
                } else {
                    match it.next() {
                        Some((j, c)) => match c {
                            '\'' => Some(&s[0..j + 2]),
                            _ => panic!("Lexer error: Unterminated character literal"),
                        },
                        None => panic!("Lexer error: Unterminated character literal"),
                    }
                }
            }
            None => panic!("Lexer error: Unterminated character literal"),
        }
    }

    // TODO: Make it return Option
    fn match_string(s: &str) -> Option<&str> {
        if !s.starts_with('\"') {
            return None;
        }
        // No other token starts with " so we can panic
        let mut it = s[1..].char_indices();
        let mut escaped = false;
        loop {
            match it.next() {
                Some((i, c)) => {
                    if c == '\n' {
                        panic!("Lexer error: String literal cannot span multiple lines");
                    }
                    if !escaped {
                        match c {
                            '\\' => escaped = true,
                            '\"' => return Some(&s[0..i + 2]),
                            '\'' => {
                                panic!("Lexer error: String literal cannot contain a single quote");
                            }
                            _ => {}
                        }
                    } else {
                        match c {
                            'n' | 't' | 'r' | '0' | '\\' | '\'' | '\"' => {}
                            _ => {
                                panic!("Lexer Error: Invalid escape sequence");
                            }
                        }
                        escaped = false;
                    }
                }
                None => {
                    panic!("Lexer Error: Unterminated character literal");
                }
            }
        }
    }

    fn match_comment(s: &str) -> Option<&str> {
        if s.starts_with("(*") {
            // Search for the closing tag in the remainder of the string
            if let Some(idx) = s[2..].find("*)") {
                // `idx` is relative to s[2..] so add two to compensate for the offset
                // and another two to include *)
                return Some(&s[0..idx + 4]);
            }
        }
        None
    }

    fn match_operator(s: &str) -> Option<&str> {
        for &operator in OPERATORS.iter() {
            if s.starts_with(operator) {
                return Some(operator);
            }
        }
        return None;
    }

    fn match_delimiter(s: &str) -> Option<&str> {
        for &delimiter in DELIMITERS.iter() {
            if s.starts_with(delimiter) {
                return Some(delimiter);
            }
        }
        return None;
    }

    fn match_whitespace(s: &str) -> Option<&str> {
        let mut it = s.char_indices();
        loop {
            match it.next() {
                Some((ind, c)) => {
                    if !is_whitespace(c) {
                        return Some(&s[0..ind]);
                    }
                }
                // This only happens in case of trailing whitespaces
                None => {
                    return Some(&s[0..]);
                }
            }
        }
    }

    /// Returns the token found at the beginning of the slice and advances it
    fn next_token(&mut self) -> Token {
        let src = self.slice;

        if src.is_empty() {
            return (TokenKind::EOF, String::new());
        }

        let mut cur_token: (TokenKind, &str) = (TokenKind::Error, "");

        let matches = [
            (Self::match_keyword(src), TokenKind::Keyword),
            (Self::match_identifier(src), TokenKind::Identifier),
            (Self::match_uint(src), TokenKind::Int),
            (Self::match_real(src), TokenKind::Real),
            (Self::match_char(src), TokenKind::Char),
            (Self::match_string(src), TokenKind::String),
            (Self::match_operator(src), TokenKind::Operator),
            (Self::match_delimiter(src), TokenKind::Delimiter),
            (Self::match_whitespace(src), TokenKind::Whitespace),
            (Self::match_comment(src), TokenKind::Comment),
        ];

        for (match_fn, kind) in matches {
            match match_fn {
                Some(lexeme) => {
                    if lexeme.len() > cur_token.1.len() {
                        cur_token = (kind, lexeme);
                    }
                }
                None => {}
            }
        }

        match cur_token.0 {
            TokenKind::Error => {
                // TODO: Return the line and column of the error
                panic!(
                    "{}:{}\n
                Lexer error: Unrecognized token starting with '{}'",
                    self.line,
                    self.column,
                    src.chars().next().unwrap()
                );
            }
            _ => {
                let lexeme = cur_token.1;
                // Update line and column tracking
                for c in lexeme.chars() {
                    if c == '\n' {
                        self.line += 1;
                        self.column = 1;
                    } else {
                        self.column += 1;
                    }
                }
                // Advance the slice past the consumed token
                self.slice = &self.slice[lexeme.len()..];
                return (cur_token.0, lexeme.to_string());
            }
        }
    }

    pub fn lex(&mut self) -> Vec<(TokenKind, String)> {
        let mut ans: Vec<(TokenKind, String)> = Vec::new();
        loop {
            let (token_kind, lexeme) = self.next_token();
            println!(
                "{}:{} TokenId: {:?}, Lexeme: {}",
                self.line, self.column, token_kind, lexeme
            );
            ans.push((token_kind.clone(), lexeme));

            match token_kind {
                TokenKind::EOF => break,
                _ => {}
            }
        }
        println!("End of file reached");
        return ans;
    }
}
