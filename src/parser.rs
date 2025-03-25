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
    lines: &[&str],
    initial_state: ParseState,
) -> Option<(Vec<Vec<Match>>, Vec<ParseState>)> {
    match filetype {
        "c" => Some(parse_with_lexer(CToken::lexer, lines, initial_state)),
        "clojure" => Some(parse_with_lexer(ClojureToken::lexer, lines, initial_state)),
        "cpp" => Some(parse_with_lexer(CppToken::lexer, lines, initial_state)),
        "csharp" => Some(parse_with_lexer(CSharpToken::lexer, lines, initial_state)),
        "dart" => Some(parse_with_lexer(DartToken::lexer, lines, initial_state)),
        "elixir" => Some(parse_with_lexer(ElixirToken::lexer, lines, initial_state)),
        "erlang" => Some(parse_with_lexer(ErlangToken::lexer, lines, initial_state)),
        "fsharp" => Some(parse_with_lexer(FSharpToken::lexer, lines, initial_state)),
        "go" => Some(parse_with_lexer(GoToken::lexer, lines, initial_state)),
        "haskell" => Some(parse_with_lexer(HaskellToken::lexer, lines, initial_state)),
        "java" => Some(parse_with_lexer(JavaToken::lexer, lines, initial_state)),
        "javascript" => Some(parse_with_lexer(
            JavaScriptToken::lexer,
            lines,
            initial_state,
        )),
        "json" | "json5" | "jsonc" => {
            Some(parse_with_lexer(JsonToken::lexer, lines, initial_state))
        }
        "kotlin" => Some(parse_with_lexer(KotlinToken::lexer, lines, initial_state)),
        "lean" => Some(parse_with_lexer(LeanToken::lexer, lines, initial_state)),
        "lua" => Some(parse_with_lexer(LuaToken::lexer, lines, initial_state)),
        "objc" => Some(parse_with_lexer(ObjCToken::lexer, lines, initial_state)),
        "ocaml" => Some(parse_with_lexer(OCamlToken::lexer, lines, initial_state)),
        "perl" => Some(parse_with_lexer(PerlToken::lexer, lines, initial_state)),
        "php" => Some(parse_with_lexer(PhpToken::lexer, lines, initial_state)),
        "python" => Some(parse_with_lexer(PythonToken::lexer, lines, initial_state)),
        "r" => Some(parse_with_lexer(RToken::lexer, lines, initial_state)),
        "ruby" => Some(parse_with_lexer(RubyToken::lexer, lines, initial_state)),
        "rust" => Some(parse_with_lexer(RustToken::lexer, lines, initial_state)),
        "scala" => Some(parse_with_lexer(ScalaToken::lexer, lines, initial_state)),
        "sh" | "bash" | "zsh" | "fish" => {
            Some(parse_with_lexer(ShellToken::lexer, lines, initial_state))
        }
        "swift" => Some(parse_with_lexer(SwiftToken::lexer, lines, initial_state)),
        "toml" => Some(parse_with_lexer(TomlToken::lexer, &lines, initial_state)),
        "typescript" => Some(parse_with_lexer(
            TypeScriptToken::lexer,
            lines,
            initial_state,
        )),
        "typst" => Some(parse_with_lexer(TypstToken::lexer, lines, initial_state)),
        "zig" => Some(parse_with_lexer(ZigToken::lexer, lines, initial_state)),
        _ => None,
    }
}

fn parse_with_lexer<'s, T>(
    mut lexer: impl FnMut(&'s str) -> Lexer<'s, T>,
    lines: &[&'s str],
    initial_state: ParseState,
) -> (Vec<Vec<Match>>, Vec<ParseState>)
where
    T: Into<Token<'s>> + Logos<'s>,
{
    let mut matches_by_line = vec![];
    let mut state_by_line = vec![];
    let mut stack = vec![];

    let mut state = initial_state;
    for line in lines.iter() {
        let mut escaped_position = None;
        let mut current_line_matches = vec![];
        let mut lexer = lexer(line);

        use {ParseState::*, Token::*};
        while let Some(token) = lexer.next() {
            let token = match token {
                Ok(token) => token.into(),
                Err(_) => continue,
            };

            let should_escape =
                matches!(escaped_position, Some(pos) if (pos == lexer.span().start));
            escaped_position = None;

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
                        col: lexer.span().start,
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
                        col: lexer.span().start,
                        closing: None,
                        stack_height: stack.len(),
                    };
                    current_line_matches.push(_match);
                }

                // TODO: continue outer loop
                (Normal, LineComment, false) => state = InLineComment,

                (Normal, BlockCommentOpen, _) => state = InBlockComment,
                (InBlockComment, BlockCommentClose, _) => state = Normal,

                (Normal, String(open), false) => state = InString(open.to_string()),
                (InString(open), String(close), false) if open == close => state = Normal,

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
        }

        // Reset state for linewise strings and comments
        state = match state {
            InString(_) | InLineComment => Normal,
            state => state,
        };

        matches_by_line.push(std::mem::take(&mut current_line_matches));
        state_by_line.push(state.clone());
    }

    (matches_by_line, state_by_line)
}
