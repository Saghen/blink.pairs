use logos::{Lexer, Logos, Source};
use mlua::{serde::Serializer, IntoLua};
use serde::Serialize;

use super::languages::*;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    text: String,
    row: usize,
    col: usize,
    closing: Option<String>,
    stack_height: usize,
}

// TODO: how do we derive this?
impl IntoLua for Match {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new(lua))
    }
}

pub fn parse_filetype(filetype: String, text: String) -> Option<Vec<Vec<Match>>> {
    match filetype.as_str() {
        "c" => Some(parse_with_lexer(CToken::lexer(&text))),
        "cpp" => Some(parse_with_lexer(CppToken::lexer(&text))),
        "csharp" => Some(parse_with_lexer(CSharpToken::lexer(&text))),
        "go" => Some(parse_with_lexer(GoToken::lexer(&text))),
        "java" => Some(parse_with_lexer(JavaToken::lexer(&text))),
        "javascript" => Some(parse_with_lexer(JavaScriptToken::lexer(&text))),
        "json" => Some(parse_with_lexer(JsonToken::lexer(&text))),
        "jsonc" => Some(parse_with_lexer(JsonToken::lexer(&text))),
        "json5" => Some(parse_with_lexer(JsonToken::lexer(&text))),
        "lua" => Some(parse_with_lexer(LuaToken::lexer(&text))),
        "php" => Some(parse_with_lexer(PhpToken::lexer(&text))),
        "python" => Some(parse_with_lexer(PythonToken::lexer(&text))),
        "ruby" => Some(parse_with_lexer(RubyToken::lexer(&text))),
        "rust" => Some(parse_with_lexer(RustToken::lexer(&text))),
        "swift" => Some(parse_with_lexer(SwiftToken::lexer(&text))),
        "typescript" => Some(parse_with_lexer(TypeScriptToken::lexer(&text))),
        "clojure" => Some(parse_with_lexer(ClojureToken::lexer(&text))),
        _ => None,
    }
}

fn parse_with_lexer<'a, T>(mut lexer: Lexer<'a, T>) -> Vec<Vec<Match>>
where
    T: Into<Token> + Logos<'a>,
    <T::Source as Source>::Slice<'a>: std::fmt::Display + AsRef<str>,
{
    let mut matches_by_line = vec![vec![]];
    let mut stack = vec![];

    let mut line_number = 0;
    let mut col_offset = 0;
    let mut escaped_position = None;

    while let Some(token) = lexer.next() {
        let token = match token {
            Ok(token) => token.into(),
            Err(_) => continue,
        };

        // Handle escaped characters
        if let Some((escaped_line, escaped_col)) = escaped_position {
            if !matches!(token, Token::NewLine) {
                escaped_position = None;
                let current_col = lexer.span().start;
                if line_number == escaped_line && current_col - 1 == escaped_col {
                    continue;
                }
            }
        }

        match token {
            Token::DelimiterOpen => {
                let _match = Match {
                    text: lexer.slice().to_string(),
                    row: line_number,
                    col: lexer.span().start - col_offset,
                    closing: Some(match lexer.slice().as_ref() {
                        "(" => ")".to_string(),
                        "[" => "]".to_string(),
                        "{" => "}".to_string(),
                        "<" => ">".to_string(),
                        char => char.to_string(),
                    }),
                    stack_height: stack.len(),
                };
                stack.push(_match.closing.clone().unwrap().clone());
                matches_by_line.last_mut().unwrap().push(_match);
            }

            Token::DelimiterClose => {
                if let Some(closing) = stack.last() {
                    if lexer.slice().as_ref() == closing {
                        stack.pop();
                    }
                }

                let _match = Match {
                    text: lexer.slice().to_string(),
                    row: line_number,
                    col: lexer.span().start - col_offset,
                    closing: None,
                    stack_height: stack.len(),
                };
                matches_by_line.last_mut().unwrap().push(_match);
            }

            Token::LineComment => {
                while let Some(token) = lexer.next() {
                    if let Ok(token) = token {
                        if matches!(token.into(), Token::NewLine) {
                            line_number += 1;
                            col_offset = lexer.span().start + 1;
                            matches_by_line.push(vec![]);
                            break;
                        }
                    }
                }
            }

            Token::BlockCommentOpen => {
                while let Some(token) = lexer.next() {
                    if let Ok(token) = token {
                        match token.into() {
                            Token::BlockCommentClose => break,
                            Token::NewLine => {
                                line_number += 1;
                                col_offset = lexer.span().start + 1;
                                matches_by_line.push(vec![]);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Token::String => {
                let end_char = lexer.slice();
                while let Some(token) = lexer.next() {
                    if let Ok(token) = token {
                        match token.into() {
                            Token::NewLine => {
                                line_number += 1;
                                col_offset = lexer.span().start + 1;
                                matches_by_line.push(vec![]);
                                break;
                            }
                            Token::String => {
                                if lexer.slice() == end_char {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // TODO: should also be in hot loops
            Token::Escape => {
                let col = lexer.span().start;
                escaped_position = Some((line_number, col));
            }

            Token::NewLine => {
                line_number += 1;
                col_offset = lexer.span().start + 1;
                matches_by_line.push(vec![]);
            }

            _ => {}
        }
    }

    matches_by_line
}

pub fn recalculate_stack_heights(matches_by_line: &mut Vec<Vec<Match>>) {
    let mut stack = vec![];

    for matches in matches_by_line {
        for match_ in matches {
            match &match_.closing {
                // Opening delimiter
                Some(closing) => {
                    match_.stack_height = stack.len();
                    stack.push(closing);
                }
                // Closing delimiter
                None => {
                    if let Some(closing) = stack.last() {
                        if *closing == &match_.text {
                            stack.pop();
                        }
                    }
                    match_.stack_height = stack.len();
                }
            }
        }
    }
}
