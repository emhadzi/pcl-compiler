use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    src: &'a str,
    tid: usize,
    tokens: Vec<Token>,
    // We will see ...
}

enum Local {}

enum Expr {}

enum Stmt {}

struct Call {
    id: String,
    exprs: Vec<Expr>,
}

struct Block {
    stmts: Vec<Stmt>,
}

struct Body {
    locals: Vec<Local>,
    block: Box<Block>,
}

struct Program {
    name: String,
    body: Box<Body>,
}

impl<'a> Parser<'a> {
    // Helper peek function
    pub fn peek_at(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx)
    }

    // Returns an option to a reference to the next token
    // or None if we have reached the last token
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.tid)
    }

    // "Consumes" current Token on the stream and advances its position by 1
    pub fn advance(&mut self) -> Option<&Token> {
        let old_tid = self.tid;
        if self.tid < self.tokens.len() {
            self.tid += 1;
        }
        // Return the reference to the consumed token
        self.peek_at(old_tid)
    }

    // Checks the next token against the one which the grammar rule expects
    // Consumes and return the token on success, returns error string on failure
    pub fn expect(
        &mut self,
        expected_tok_kind: &TokenKind,
        expected_str: Option<&str>,
    ) -> Result<&Token, String> {
        let (curr_tok_kind, curr_tok_str) = self.peek().unwrap();
        if curr_tok_kind != expected_tok_kind {
            return Err(format!(
                "Syntax error: Expected '{:?}', found '{:?}'",
                expected_tok_kind, curr_tok_kind
            ));
        }
        match expected_str {
            Some(expected_str) => {
                if curr_tok_str.as_str() == expected_str {
                    Ok(self.advance().unwrap())
                } else {
                    Err(format!(
                        "Syntax error: Expected '{:?}', but found '{:?}'",
                        expected_str,
                        curr_tok_str.as_str()
                    ))
                }
            }
            None => Ok(self.advance().unwrap()),
        }
    }

    pub fn parse_binop(tok: &Token) -> Option<&str> {
        match tok.1.as_str() {
            op @ ("+" | "-" | "*" | "/" | "div" | "mod" | "or" | "and" | "=" | "<>" | "<"
            | "<=" | ">" | ">=") => Some(op),
            _ => None,
        }
    }

    pub fn parse_unop(tok: &Token) -> Option<&str> {
        match tok.1.as_str() {
            op @ ("not" | "+" | "-") => Some(op),
            _ => None,
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        self.expect(TokenKind::Keyword, Some("program"))?;
        let name = self.expect(TokenKind::Identifier, None)?;
        self.expect(TokenKind::Delimiter, Some(";"))?;
        let body = Box::new(self.parse_body()?);
        self.expect(TokenKind::Delimiter, Some("."))?;

        return Ok(Program { name: name.1, body });
    }

    fn parse_body(&mut self) -> Result<Body, String> {
        while self.peek()?.1 == "(" {
            self.advance();
            let cur_local = Box::new(self.parse_local()?);
            let
        }
        
    }
    
    fn parse_local(&mut self) -> Result<Local, String> {
        
    }
    
}
