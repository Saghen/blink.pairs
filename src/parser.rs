use logos::{Lexer, Logos};
use mlua::{serde::Serializer, IntoLua};
use serde::Serialize;

use super::languages::*;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub text: String,
    pub col: usize,
    pub closing: Option<String>,
    pub stack_height: usize,
}

// TODO: how do we derive this?
impl IntoLua for Match {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new(lua))
    }
}

#[derive(Debug, Clone)]
pub enum ParseState {
    Normal,
    InLineComment,
    InBlockComment,
    InString(String),
    InBlockStringSymmetric(String),
    InBlockString,
}

pub fn parse_filetype(
    filetype: &str,
    text: &str,
    initial_state: ParseState,
) -> Option<(Vec<Vec<Match>>, Vec<ParseState>)> {
    match filetype {
        "c" => Some(parse_with_lexer(CToken::lexer(text), initial_state)),
        "cpp" => Some(parse_with_lexer(CppToken::lexer(text), initial_state)),
        "csharp" => Some(parse_with_lexer(CSharpToken::lexer(text), initial_state)),
        "dart" => Some(parse_with_lexer(DartToken::lexer(text), initial_state)),
        "go" => Some(parse_with_lexer(GoToken::lexer(text), initial_state)),
        "haskell" => Some(parse_with_lexer(HaskellToken::lexer(text), initial_state)),
        "java" => Some(parse_with_lexer(JavaToken::lexer(text), initial_state)),
        "javascript" => Some(parse_with_lexer(
            JavaScriptToken::lexer(text),
            initial_state,
        )),
        "json" => Some(parse_with_lexer(JsonToken::lexer(text), initial_state)),
        "jsonc" => Some(parse_with_lexer(JsonToken::lexer(text), initial_state)),
        "json5" => Some(parse_with_lexer(JsonToken::lexer(text), initial_state)),
        "lean" => Some(parse_with_lexer(LeanToken::lexer(text), initial_state)),
        "lua" => Some(parse_with_lexer(LuaToken::lexer(text), initial_state)),
        "php" => Some(parse_with_lexer(PhpToken::lexer(text), initial_state)),
        "python" => Some(parse_with_lexer(PythonToken::lexer(text), initial_state)),
        "ruby" => Some(parse_with_lexer(RubyToken::lexer(text), initial_state)),
        "rust" => Some(parse_with_lexer(RustToken::lexer(text), initial_state)),
        "swift" => Some(parse_with_lexer(SwiftToken::lexer(text), initial_state)),
        "toml" => Some(parse_with_lexer(TomlToken::lexer(text), initial_state)),
        "typescript" => Some(parse_with_lexer(
            TypeScriptToken::lexer(text),
            initial_state,
        )),
        "clojure" => Some(parse_with_lexer(ClojureToken::lexer(text), initial_state)),
        "typst" => Some(parse_with_lexer(TypstToken::lexer(text), initial_state)),
        "zig" => Some(parse_with_lexer(ZigToken::lexer(text), initial_state)),
        _ => None,
    }
}

fn parse_with_lexer<'s, T>(
    mut lexer: Lexer<'s, T>,
    initial_state: ParseState,
) -> (Vec<Vec<Match>>, Vec<ParseState>)
where
    T: Into<Token<'s>> + Logos<'s>,
{
    let mut matches_by_line = vec![];
    let mut state_by_line = vec![];
    let mut stack = vec![];

    let mut current_line_matches = vec![];
    let mut col_offset = 0;
    let mut escaped_position = None;

    let mut state = initial_state;
    while let Some(token) = lexer.next() {
        let token = match token {
            Ok(token) => token.into(),
            Err(_) => continue,
        };

        let should_escape = matches!(escaped_position, Some(pos) if (pos == lexer.span().start));
        escaped_position = None;

        use {ParseState::*, Token::*};
        match (&state, token, should_escape) {
            (Normal, DelimiterOpen(open), false) => {
                let closing = match open {
                    "(" => ")",
                    "[" => "]",
                    "{" => "}",
                    "<" => ">",
                    char => char,
                };
                let _match = Match {
                    text: open.to_string(),
                    col: lexer.span().start - col_offset,
                    closing: Some(closing.to_string()),
                    stack_height: stack.len(),
                };
                stack.push(closing);
                current_line_matches.push(_match);
            }
            (Normal, DelimiterClose(close), false) => {
                if let Some(closing) = stack.last() {
                    if close == *closing {
                        stack.pop();
                    }
                }

                let _match = Match {
                    text: close.to_string(),
                    col: lexer.span().start - col_offset,
                    closing: None,
                    stack_height: stack.len(),
                };
                current_line_matches.push(_match);
            }

            (Normal, LineComment, false) => state = InLineComment,
            (InLineComment, NewLine, _) => state = Normal,

            (Normal, BlockCommentOpen, _) => state = InBlockComment,
            (InBlockComment, BlockCommentClose, _) => state = Normal,

            (Normal, String(open), false) => state = InString(open.to_string()),
            (InString(open), String(close), false) if open == close => state = Normal,
            (InString(_), NewLine, false) => state = Normal,

            (Normal, BlockStringSymmetric(open), _) => {
                state = InBlockStringSymmetric(open.to_string())
            }
            (InBlockStringSymmetric(open), BlockStringSymmetric(close), _) if open == close => {
                state = Normal
            }

            (Normal, BlockStringOpen, _) => state = InBlockString,
            (InBlockString, BlockStringClose, _) => state = Normal,

            (_, Escape, false) => escaped_position = Some(lexer.span().end),
            _ => {}
        }

        if matches!(token, NewLine) {
            col_offset = lexer.span().end;
            matches_by_line.push(std::mem::take(&mut current_line_matches));
            state_by_line.push(state.clone());
        }
    }

    matches_by_line.push(current_line_matches);
    state_by_line.push(state);
    (matches_by_line, state_by_line)
}
