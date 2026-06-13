use crate::token::{Token, TokenKind};

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut indent_stack = vec![0usize];

    let mut i = 0;
    let mut line = 1;
    let mut column = 1;
    let mut at_line_start = true;
    let mut line_ended_with_colon = false;

    while i < chars.len() {
        if at_line_start {
            let mut indent = 0;

            while i < chars.len() {
                match chars[i] {
                    ' ' => {
                        indent += 1;
                        i += 1;
                        column += 1;
                    }
                    '\t' => {
                        indent += 4;
                        i += 1;
                        column += 1;
                    }
                    '\r' => {
                        i += 1;
                    }
                    _ => break,
                }
            }

            if i >= chars.len() {
                break;
            }

            if chars[i] != '\n' {
                let current_indent = *indent_stack.last().unwrap();

                if line_ended_with_colon {
                    if indent <= current_indent {
                        return Err(format!(
                            "AUIG Error: expected indented block after ':' at line {}, column {}",
                            line, column
                        ));
                    }

                    indent_stack.push(indent);
                    tokens.push(Token::new(TokenKind::Indent, line, 1));
                } else if indent < current_indent {
                    while indent < *indent_stack.last().unwrap() {
                        indent_stack.pop();
                        tokens.push(Token::new(TokenKind::Dedent, line, column));
                    }

                    if indent != *indent_stack.last().unwrap() && indent_stack.len() > 1 {
                        return Err(format!(
                            "AUIG Error: indentation does not match any previous block at line {}, column {}",
                            line, column
                        ));
                    }
                } else if indent > current_indent && indent_stack.len() > 1 {
                    return Err(format!(
                        "AUIG Error: unexpected indentation at line {}, column {}",
                        line, column
                    ));
                }

                line_ended_with_colon = false;
                at_line_start = false;
            }
        }

        let ch = chars[i];

        match ch {
            ':' => {
                tokens.push(Token::new(TokenKind::Colon, line, column));
                line_ended_with_colon = true;
                i += 1;
                column += 1;
            }

            ' ' | '\t' | '\r' => {
                i += 1;
                column += 1;
            }

            '\n' => {
                tokens.push(Token::new(TokenKind::Newline, line, column));
                i += 1;
                line += 1;
                column = 1;
                at_line_start = true;
            }

            '{' => {
                tokens.push(Token::new(TokenKind::LBrace, line, column));
                i += 1;
                column += 1;
            }

            '}' => {
                tokens.push(Token::new(TokenKind::RBrace, line, column));
                i += 1;
                column += 1;
            }

            '"' => {
                let start_column = column;
                i += 1;
                column += 1;

                let mut value = String::new();

                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\n' {
                        return Err(format!(
                            "AUIG Error: string was not closed at line {}, column {}",
                            line, start_column
                        ));
                    }

                    value.push(chars[i]);
                    i += 1;
                    column += 1;
                }

                if i >= chars.len() {
                    return Err(format!(
                        "AUIG Error: string was not closed at line {}, column {}",
                        line, start_column
                    ));
                }

                i += 1;
                column += 1;

                tokens.push(Token::new(TokenKind::String(value), line, start_column));
            }

            c if is_ident_start(c) => {
                let start_column = column;
                let mut value = String::new();

                while i < chars.len() && is_ident_char(chars[i]) {
                    value.push(chars[i]);
                    i += 1;
                    column += 1;
                }

                tokens.push(Token::new(TokenKind::Ident(value), line, start_column));
            }

            _ => {
                return Err(format!(
                    "AUIG Error: unexpected character '{}' at line {}, column {}",
                    ch, line, column
                ));
            }
        }
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::new(TokenKind::Dedent, line, column));
    }

    tokens.push(Token::new(TokenKind::Eof, line, column));
    Ok(tokens)
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}
