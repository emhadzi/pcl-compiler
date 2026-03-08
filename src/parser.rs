use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    src: &'a str,
    tid: usize,
    tokens: Vec<Token>,
    // We will see ...
}

enum Local {}

enum Header {
    Procedure {
        name: String,
        args: Vec<Box<Formal>>,
    },
    Function {
        name: String,
        args: Vec<Box<Formal>>,
        ret: Box<Type>,
    },
}

struct Formal {}

enum Type {}

enum Expr {}

enum Stmt {}

struct Call {
    id: String,
    exprs: Vec<Box<Expr>>,
}

struct Block {
    stmts: Vec<Stmt>,
}

struct Body {
    locals: Vec<Box<Local>>,
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
        expected_tok_kind: TokenKind,
        expected_str: Option<&str>,
    ) -> Result<Token, String> {
        let (curr_tok_kind, curr_tok_str) = self.peek().unwrap();
        let curr_tok_kind = curr_tok_kind.clone();
        let curr_tok_str = curr_tok_str.clone();

        if curr_tok_kind != expected_tok_kind {
            return Err(format!(
                "Syntax error: Expected '{:?}', found '{:?}'",
                expected_tok_kind, curr_tok_kind
            ));
        }
        match expected_str {
            Some(expected_str) => {
                if curr_tok_str.as_str() == expected_str {
                    Ok(self.advance().unwrap().clone())
                } else {
                    Err(format!(
                        "Syntax error: Expected '{:?}', but found '{:?}'",
                        expected_str,
                        curr_tok_str.as_str()
                    ))
                }
            }
            None => Ok(self.advance().unwrap().clone()),
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
        let mut locals: Vec<Box<Local>> = Vec::new();
        loop {
            match self.parse_local() {
                Ok(res) => {
                    locals.push(Box::new(res));
                }
                Err(_) => {
                    break;
                }
            }
        }
        let block = Box::new(self.parse_block()?);

        return Ok(Body {
            locals: locals,
            block: block,
        });
    }

    fn parse_local(&mut self) -> Result<Local, String> { todo!() }

    fn parse_header(&mut self) -> Result<Header, String> {
        let mut is_func = false;
        match self.advance() {
            Some((TokenKind::Keyword, s)) => {
                if s == "function" {
                    is_func = true;
                } else if s != "procedure" {
                    return Err(format!(
                        "Syntax error: expected \"function\" or \"procedure\", found \"{}\"",
                        s
                    ));
                }
            }
            _ => {
                return Err("Syntax error: expected \"function\" or \"procedure\"".to_string());
            }
        }

        let name = self.expect(TokenKind::Identifier, None)?;
        self.expect(TokenKind::Delimiter, Some("("))?;

        let mut args: Vec<Box<Formal>> = Vec::new();
        args.push(Box::new(self.parse_formal()?));

        loop {
            if let Err(_) = self.expect(TokenKind::Delimiter, Some(";")) {
                break;
            }
            match self.parse_formal() {
                Ok(formal) => {
                    args.push(Box::new(formal));
                }
                Err(_) => {
                    break;
                }
            }
        }

        self.expect(TokenKind::Delimiter, Some(")"))?;

        if is_func {
            self.expect(TokenKind::Delimiter, Some(":"))?;
            let ret = Box::new(self.parse_type()?);
            return Ok(Header::Function {
                name: name.1,
                args,
                ret,
            });
        }

        return Ok(Header::Procedure {
            name: name.1,
            args,
        });
    }

    fn parse_block(&mut self) -> Result<Block, String> { todo!() }
    fn parse_formal(&mut self) -> Result<Formal, String> { todo!() }
    fn parse_type(&mut self) -> Result<Type, String> { todo!() }

    fn parse_call(&mut self) -> Result<Call, String> {
        let id = self.expect(TokenKind::Identifier, None)?;
        self.expect(TokenKind::Delimiter, Some("("))?;
        let mut exprs: Vec<Box<Expr>> = Vec::new();
        loop {
            match self.parse_expr() {
                Ok(expr) => {
                    exprs.push(Box::new(expr));
                }
                Err(_) => {
                    break;
                }
            }
            match self.expect(TokenKind::Delimiter, Some(",")) {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
        self.expect(TokenKind::Delimiter, Some(")"));

        return Ok(Call {
            id: id.1,
            exprs: exprs,
        });
    }
}
