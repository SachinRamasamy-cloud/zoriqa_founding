use crate::ast::{Node, Program, Property};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        self.skip_newlines();

        let keyword = self.expect_ident()?;
        if keyword != "page" {
            return self.error("expected 'page' at the start of the file");
        }

        let page_name = self.expect_ident()?;

        let mut title = None;
        let mut children = Vec::new();

        if self.check_colon() {
            self.expect_colon_block_start()?;

            self.parse_page_body(&mut title, &mut children)?;
            self.expect_dedent()?;
        } else {
            self.skip_newlines();
            self.expect_lbrace()?;

            self.parse_page_body(&mut title, &mut children)?;
            self.expect_rbrace()?;
        }

        Ok(Program {
            page_name,
            title,
            children,
        })
    }

    fn parse_page_body(
        &mut self,
        title: &mut Option<String>,
        children: &mut Vec<Node>,
    ) -> Result<(), String> {
        loop {
            self.skip_newlines();

            if self.check_dedent() || self.check_rbrace() || self.check_eof() {
                break;
            }

            if self.check_ident("title") {
                self.advance();
                *title = Some(self.expect_string()?);
                self.consume_until_line_end();
                continue;
            }

            children.push(self.parse_node()?);
        }

        Ok(())
    }

    fn parse_node(&mut self) -> Result<Node, String> {
        let kind = self.expect_ident()?;

        let mut value = None;
        let mut flags = Vec::new();
        let mut props = Vec::new();
        let mut children = Vec::new();

        if self.check_string() {
            value = Some(self.expect_string()?);
        }

        while !self.check_lbrace()
            && !self.check_colon()
            && !self.check_rbrace()
            && !self.check_dedent()
            && !self.check_newline()
            && !self.check_eof()
        {
            match self.peek().kind.clone() {
                TokenKind::Ident(name) => {
                    self.advance();

                    if self.check_string() {
                        let value = self.expect_string()?;
                        props.push(Property { name, value });
                    } else {
                        flags.push(name);
                    }
                }
                _ => return self.error("expected style keyword, property, or block"),
            }
        }

        if self.check_lbrace() {
            children = self.parse_block()?;
        } else if self.check_colon() {
            children = self.parse_indent_block()?;
        }

        Ok(Node {
            kind,
            value,
            flags,
            props,
            children,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Node>, String> {
        self.expect_lbrace()?;

        let mut children = Vec::new();

        loop {
            self.skip_newlines();

            if self.check_rbrace() || self.check_eof() {
                break;
            }

            children.push(self.parse_node()?);
        }

        self.expect_rbrace()?;
        Ok(children)
    }

    fn parse_indent_block(&mut self) -> Result<Vec<Node>, String> {
        self.expect_colon_block_start()?;

        let mut children = Vec::new();

        loop {
            self.skip_newlines();

            if self.check_dedent() || self.check_eof() {
                break;
            }

            children.push(self.parse_node()?);
        }

        self.expect_dedent()?;
        Ok(children)
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.peek().kind.clone() {
            TokenKind::Ident(value) => {
                self.advance();
                Ok(value)
            }
            _ => self.error("expected identifier"),
        }
    }

    fn expect_string(&mut self) -> Result<String, String> {
        match self.peek().kind.clone() {
            TokenKind::String(value) => {
                self.advance();
                Ok(value)
            }
            _ => self.error("expected string"),
        }
    }

    fn expect_lbrace(&mut self) -> Result<(), String> {
        if self.check_lbrace() {
            self.advance();
            Ok(())
        } else {
            self.error("expected '{'")
        }
    }

    fn expect_rbrace(&mut self) -> Result<(), String> {
        if self.check_rbrace() {
            self.advance();
            Ok(())
        } else {
            self.error("expected '}'")
        }
    }

    fn expect_colon_block_start(&mut self) -> Result<(), String> {
        self.expect_colon()?;

        if self.check_newline() {
            self.advance();
        } else {
            return self.error("expected new line after ':'");
        }

        self.skip_newlines();
        self.expect_indent()
    }

    fn expect_colon(&mut self) -> Result<(), String> {
        if self.check_colon() {
            self.advance();
            Ok(())
        } else {
            self.error("expected ':'")
        }
    }

    fn expect_indent(&mut self) -> Result<(), String> {
        if self.check_indent() {
            self.advance();
            Ok(())
        } else {
            self.error("expected indented block after ':'")
        }
    }

    fn expect_dedent(&mut self) -> Result<(), String> {
        if self.check_dedent() {
            self.advance();
            Ok(())
        } else {
            self.error("expected end of indented block")
        }
    }

    fn skip_newlines(&mut self) {
        while self.check_newline() {
            self.advance();
        }
    }

    fn consume_until_line_end(&mut self) {
        while !self.check_newline()
            && !self.check_rbrace()
            && !self.check_dedent()
            && !self.check_eof()
        {
            self.advance();
        }

        self.skip_newlines();
    }

    fn check_ident(&self, name: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(value) if value == name)
    }

    fn check_string(&self) -> bool {
        matches!(self.peek().kind, TokenKind::String(_))
    }

    fn check_lbrace(&self) -> bool {
        matches!(self.peek().kind, TokenKind::LBrace)
    }

    fn check_colon(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Colon)
    }

    fn check_rbrace(&self) -> bool {
        matches!(self.peek().kind, TokenKind::RBrace)
    }

    fn check_indent(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Indent)
    }

    fn check_dedent(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Dedent)
    }

    fn check_newline(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Newline)
    }

    fn check_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn error<T>(&self, message: &str) -> Result<T, String> {
        let token = self.peek();

        Err(format!(
            "AUIG Error: {} at line {}, column {}",
            message, token.line, token.column
        ))
    }
}
