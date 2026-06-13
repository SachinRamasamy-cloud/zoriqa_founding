use crate::ast::{
    ComponentDecl, ElementNode, LayoutDecl, Node, PageDecl, Program, ResolvedFlags,
    StyleOverrides, TextNode, ThemeDecl, Tone, Variant, StateFlag, TopLevel, Value, ComponentNode
};
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
        let mut declarations = Vec::new();
        self.skip_newlines();

        while !self.check_eof() {
            let line = self.peek().line;
            let keyword = self.expect_ident()?;
            match keyword.as_str() {
                "import" => {
                    let path = self.expect_string()?;
                    declarations.push(TopLevel::Import(path));
                    self.consume_until_line_end();
                }
                "theme" => {
                    let mut name = None;
                    if self.check_string() {
                        name = Some(self.expect_string()?);
                    } else if !self.check_colon() && !self.check_lbrace() {
                        name = Some(self.expect_ident()?);
                    }

                    let mut variables = std::collections::HashMap::new();
                    if self.check_colon() {
                        self.expect_colon_block_start()?;
                        self.parse_theme_body(&mut variables)?;
                        self.expect_dedent()?;
                    } else if self.check_lbrace() {
                        self.expect_lbrace()?;
                        self.parse_theme_body(&mut variables)?;
                        self.expect_rbrace()?;
                    } else {
                        return self.error("expected ':' or '{' after theme");
                    }
                    declarations.push(TopLevel::Theme(ThemeDecl { name, variables }));
                }
                "layout" => {
                    let name = self.expect_ident()?;
                    let children = self.parse_children_block()?;
                    declarations.push(TopLevel::Layout(LayoutDecl { name, children, line }));
                }
                "page" => {
                    let name = self.expect_ident()?;
                    let mut layout = None;
                    if self.check_ident("layout") {
                        self.advance();
                        layout = Some(self.expect_ident()?);
                    }

                    let mut title = None;
                    let mut children = Vec::new();
                    if self.check_colon() {
                        self.expect_colon_block_start()?;
                        self.parse_page_body(&mut title, &mut children)?;
                        self.expect_dedent()?;
                    } else if self.check_lbrace() {
                        self.expect_lbrace()?;
                        self.parse_page_body(&mut title, &mut children)?;
                        self.expect_rbrace()?;
                    } else {
                        return self.error("expected ':' or '{' after page");
                    }
                    declarations.push(TopLevel::Page(PageDecl { name, layout, title, children, line }));
                }
                "component" => {
                    let name = self.expect_ident()?;
                    let mut params = Vec::new();
                    while !self.check_colon() && !self.check_lbrace() && !self.check_eof() && !self.check_newline() {
                        params.push(self.expect_ident()?);
                    }
                    let children = self.parse_children_block()?;
                    declarations.push(TopLevel::Component(ComponentDecl { name, params, children, line }));
                }
                _ => {
                    return self.error(&format!("unexpected top-level keyword '{}'", keyword));
                }
            }
            self.skip_newlines();
        }

        Ok(Program { declarations })
    }

    fn parse_theme_body(&mut self, variables: &mut std::collections::HashMap<String, String>) -> Result<(), String> {
        loop {
            self.skip_newlines();
            if self.check_dedent() || self.check_rbrace() || self.check_eof() {
                break;
            }
            let key = self.expect_ident()?;
            let value = if self.check_string() {
                self.expect_string()?
            } else {
                self.expect_ident()?
            };
            variables.insert(key, value);
            self.consume_until_line_end();
        }
        Ok(())
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
        self.skip_newlines();
        let line = self.peek().line;

        if self.check_string() {
            let content = self.expect_string()?;
            self.consume_until_line_end();
            return Ok(Node::Text(TextNode { content, line }));
        }

        let kind = self.expect_ident()?;

        if kind == "slot" {
            self.consume_until_line_end();
            return Ok(Node::Element(ElementNode {
                tag: "slot".to_string(),
                args: Vec::new(),
                props: std::collections::HashMap::new(),
                flags: Vec::new(),
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            }));
        }

        if kind == "style" {
            let mut style_overrides = StyleOverrides::default();
            if self.check_colon() {
                self.expect_colon_block_start()?;
                self.parse_style_body(&mut style_overrides)?;
                self.expect_dedent()?;
            } else if self.check_lbrace() {
                self.expect_lbrace()?;
                self.parse_style_body(&mut style_overrides)?;
                self.expect_rbrace()?;
            }
            return Ok(Node::Element(ElementNode {
                tag: "style".to_string(),
                args: Vec::new(),
                props: std::collections::HashMap::new(),
                flags: Vec::new(),
                children: Vec::new(),
                style: style_overrides,
                line,
            }));
        }

        let mut args = Vec::new();
        let mut props = std::collections::HashMap::new();
        let mut flags = Vec::new();
        let mut children = Vec::new();
        let mut style = StyleOverrides::default();

        while !self.check_lbrace()
            && !self.check_colon()
            && !self.check_rbrace()
            && !self.check_dedent()
            && !self.check_newline()
            && !self.check_eof()
        {
            match self.peek().kind.clone() {
                TokenKind::String(val) => {
                    self.advance();
                    args.push(Value::String(val));
                }
                TokenKind::Ident(name) => {
                    self.advance();
                    let has_value = !self.check_lbrace()
                        && !self.check_colon()
                        && !self.check_rbrace()
                        && !self.check_dedent()
                        && !self.check_newline()
                        && !self.check_eof();

                    if has_value {
                        match self.peek().kind.clone() {
                            TokenKind::String(val) => {
                                self.advance();
                                props.insert(name, Value::String(val));
                            }
                            TokenKind::Ident(val) => {
                                self.advance();
                                props.insert(name, Value::Ident(val));
                            }
                            _ => {
                                flags.push(name);
                            }
                        }
                    } else {
                        flags.push(name);
                    }
                }
                _ => return self.error("expected argument, property, flag, or block"),
            }
        }

        let has_child_block = self.check_lbrace() || self.check_colon();
        if has_child_block {
            let raw_children = if self.check_lbrace() {
                self.parse_block()?
            } else {
                self.parse_indent_block()?
            };

            for child in raw_children {
                match child {
                    Node::Element(ref e) if e.tag == "style" => {
                        style = e.style.clone();
                    }
                    _ => {
                        children.push(child);
                    }
                }
            }
        }

        if is_core_element(&kind) {
            Ok(Node::Element(ElementNode {
                tag: kind,
                args,
                props,
                flags,
                children,
                style,
                line,
            }))
        } else {
            let resolved_flags = resolve_flags(&flags);
            Ok(Node::Component(ComponentNode {
                name: kind,
                args,
                props,
                flags: resolved_flags,
                children,
                style,
                line,
            }))
        }
    }

    fn parse_style_body(&mut self, style: &mut StyleOverrides) -> Result<(), String> {
        loop {
            self.skip_newlines();
            if self.check_dedent() || self.check_rbrace() || self.check_eof() {
                break;
            }
            let key = self.expect_ident()?;
            let val = if self.check_string() {
                self.expect_string()?
            } else {
                self.expect_ident()?
            };
            match key.as_str() {
                "bg" => style.bg = Some(val),
                "text" => style.text = Some(val),
                "border" => style.border = Some(val),
                "radius" => style.radius = Some(val),
                "shadow" => style.shadow = Some(val),
                "padding" => { style.custom.insert("padding".to_string(), val); }
                "margin" => { style.custom.insert("margin".to_string(), val); }
                "width" => { style.custom.insert("width".to_string(), val); }
                "height" => { style.custom.insert("height".to_string(), val); }
                "align" => { style.custom.insert("align".to_string(), val); }
                "justify" => { style.custom.insert("justify".to_string(), val); }
                _ => {
                    let allowed_keys = vec![
                        "bg", "text", "border", "radius", "shadow", "padding",
                        "margin", "width", "height", "align", "justify"
                    ];
                    let mut closest = "";
                    let mut min_dist = usize::MAX;
                    for k in &allowed_keys {
                        let dist = Self::levenshtein_distance(&key, k);
                        if dist < min_dist {
                            min_dist = dist;
                            closest = k;
                        }
                    }
                    return Err(format!(
                        "AUIG Error: Unknown style property \"{}\".\n\nDid you mean:\n\"{}\"?\n\n(at line {})",
                        key, closest, self.peek().line
                    ));
                }
            }
            self.consume_until_line_end();
        }
        Ok(())
    }

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                matrix[i - 1][j] + 1,
                std::cmp::min(matrix[i][j - 1] + 1, matrix[i - 1][j - 1] + cost)
            );
        }
    }
    matrix[len1][len2]
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

    fn parse_children_block(&mut self) -> Result<Vec<Node>, String> {
        if self.check_colon() {
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
        } else if self.check_lbrace() {
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
        } else {
            self.consume_until_line_end();
            Ok(Vec::new())
        }
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

fn is_core_element(tag: &str) -> bool {
    matches!(
        tag,
        "view"
            | "section"
            | "row"
            | "column"
            | "col"
            | "box"
            | "h1"
            | "h2"
            | "h3"
            | "p"
            | "text"
            | "link"
            | "action"
            | "button"
            | "image"
            | "input"
            | "textarea"
            | "select"
            | "form"
            | "table"
            | "thead"
            | "tbody"
            | "tr"
            | "td"
            | "list"
            | "item"
            | "slot"
            | "subtitle"
            | "desc"
            | "message"
            | "icon"
            | "card"
            | "center"
            | "heading"
            | "btn"
            | "a"
            | "span"
    )
}

fn resolve_flags(flags: &[String]) -> ResolvedFlags {
    let mut resolved = ResolvedFlags::default();
    for flag in flags {
        let flag_lower = flag.to_lowercase();
        match flag_lower.as_str() {
            "primary" => resolved.tone = Some(Tone::Primary),
            "success" => resolved.tone = Some(Tone::Success),
            "warning" => resolved.tone = Some(Tone::Warning),
            "danger" | "error" => resolved.tone = Some(Tone::Danger),
            "info" => resolved.tone = Some(Tone::Info),
            "neutral" | "gray" => resolved.tone = Some(Tone::Neutral),

            "soft" => resolved.variant = Some(Variant::Soft),
            "solid" => resolved.variant = Some(Variant::Solid),
            "outline" => resolved.variant = Some(Variant::Outline),
            "dark" => resolved.variant = Some(Variant::Dark),
            "light" => resolved.variant = Some(Variant::Light),
            "minimal" => resolved.variant = Some(Variant::Minimal),

            "sticky" => resolved.state.push(StateFlag::Sticky),
            "popular" => resolved.state.push(StateFlag::Popular),
            "disabled" => resolved.state.push(StateFlag::Disabled),
            "loading" => resolved.state.push(StateFlag::Loading),
            "shadow" => resolved.state.push(StateFlag::Shadow),

            _ => resolved.custom.push(flag.clone()),
        }
    }
    resolved
}
