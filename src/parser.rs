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

#[derive(Debug, Clone, Copy)]
enum ParseState<'a> {
    Normal,
    InLineComment,
    InBlockComment,
    InString(&'a str),
    InBlockString,
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
        "typst" => Some(parse_with_lexer(TypstToken::lexer(&text))),
        _ => None,
    }
}

fn parse_with_lexer<'a, T>(mut lexer: Lexer<'a, T>) -> Vec<Vec<Match>>
where
    T: Into<Token> + Logos<'a>,
    T::Source: Source<Slice<'a> = &'a str>,
{
    let mut matches_by_line = vec![vec![]];
    let mut stack = vec![];

    let mut line_number = 0;
    let mut col_offset = 0;
    let mut escaped_position = None;

    let mut state = ParseState::Normal;
    while let Some(token) = lexer.next() {
        let token = match token {
            Ok(token) => token.into(),
            Err(_) => continue,
        };

        let should_escape = matches!(escaped_position, Some(pos) if (pos == lexer.span().start));
        escaped_position = None;

        use {ParseState::*, Token::*};
        match (state, &token, should_escape) {
            (Normal, DelimiterOpen, false) => {
                let _match = Match {
                    text: lexer.slice().to_string(),
                    row: line_number,
                    col: lexer.span().start - col_offset,
                    closing: Some(match lexer.slice() {
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
            (Normal, DelimiterClose, false) => {
                if let Some(closing) = stack.last() {
                    if lexer.slice() == closing {
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
            (Normal, String, false) => state = InString(lexer.slice()),
            (InString(open), String, false) if open == lexer.slice() => state = Normal,
            (InString(_), NewLine, _) => state = Normal,

            (Normal, LineComment, false) => state = InLineComment,
            (InLineComment, NewLine, _) => state = Normal,

            (Normal, BlockCommentOpen, _) => state = InBlockComment,
            (InBlockComment, BlockCommentClose, _) => state = Normal,

            (Normal, BlockStringOpen, _) => state = InBlockString,
            (InBlockComment, BlockStringClose, _) => state = Normal,

            (_, Escape, false) => escaped_position = Some(lexer.span().end),
            _ => {}
        }

        if matches!(token, NewLine) {
            line_number += 1;
            col_offset = lexer.span().end;
            matches_by_line.push(vec![]);
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
