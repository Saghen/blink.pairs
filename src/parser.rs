use logos::{Lexer, Logos};
use mlua::{serde::Serializer, IntoLua, SerializeOptions};
use serde::Serialize;

use super::languages::*;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    #[serde(skip)]
    pub type_: TokenType,

    pub text: &'static str,
    pub closing: Option<&'static str>,

    pub col: usize,
    pub stack_height: Option<usize>,
}

impl Match {
    pub fn with_line(&self, line: usize) -> MatchWithLine {
        MatchWithLine {
            type_: self.type_,
            text: self.text,
            col: self.col,
            closing: self.closing,
            stack_height: self.stack_height,
            line,
        }
    }
}

impl IntoLua for Match {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new_with_options(
            lua,
            SerializeOptions::new().serialize_none_to_null(false),
        ))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchWithLine {
    #[serde(skip)]
    pub type_: TokenType,

    pub text: &'static str,
    pub closing: Option<&'static str>,

    pub line: usize,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl IntoLua for MatchWithLine {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new_with_options(
            lua,
            SerializeOptions::new().serialize_none_to_null(false),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum ParseState {
    Normal,
    InBlockComment(&'static str),
    InBlockString(&'static str),
}

pub fn parse_filetype(
    filetype: &str,
    lines: &[&str],
    initial_state: ParseState,
) -> Option<(Vec<Vec<Match>>, Vec<ParseState>)> {
    Some(match filetype {
        "c" => parse_with_lexer(CToken::lexer, lines, initial_state),
        "clojure" => parse_with_lexer(ClojureToken::lexer, lines, initial_state),
        "cpp" => parse_with_lexer(CppToken::lexer, lines, initial_state),
        "csharp" => parse_with_lexer(CSharpToken::lexer, lines, initial_state),
        "dart" => parse_with_lexer(DartToken::lexer, lines, initial_state),
        "elixir" => parse_with_lexer(ElixirToken::lexer, lines, initial_state),
        "erlang" => parse_with_lexer(ErlangToken::lexer, lines, initial_state),
        "fsharp" => parse_with_lexer(FSharpToken::lexer, lines, initial_state),
        "go" => parse_with_lexer(GoToken::lexer, lines, initial_state),
        "haskell" => parse_with_lexer(HaskellToken::lexer, lines, initial_state),
        "java" => parse_with_lexer(JavaToken::lexer, lines, initial_state),
        "javascript" | "typescript" | "javascriptreact" | "typescriptreact" => {
            parse_with_lexer(JavaScriptToken::lexer, lines, initial_state)
        }
        "json" | "json5" | "jsonc" => parse_with_lexer(JsonToken::lexer, lines, initial_state),
        "kotlin" => parse_with_lexer(KotlinToken::lexer, lines, initial_state),
        "tex" | "bib" => parse_with_lexer(LatexToken::lexer, lines, initial_state),
        "lean" => parse_with_lexer(LeanToken::lexer, lines, initial_state),
        "lua" => parse_with_lexer(LuaToken::lexer, lines, initial_state),
        "objc" => parse_with_lexer(ObjCToken::lexer, lines, initial_state),
        "ocaml" => parse_with_lexer(OCamlToken::lexer, lines, initial_state),
        "perl" => parse_with_lexer(PerlToken::lexer, lines, initial_state),
        "php" => parse_with_lexer(PhpToken::lexer, lines, initial_state),
        "python" => parse_with_lexer(PythonToken::lexer, lines, initial_state),
        "r" => parse_with_lexer(RToken::lexer, lines, initial_state),
        "ruby" => parse_with_lexer(RubyToken::lexer, lines, initial_state),
        "rust" => parse_with_lexer(RustToken::lexer, lines, initial_state),
        "scala" => parse_with_lexer(ScalaToken::lexer, lines, initial_state),
        "sh" | "bash" | "zsh" | "fish" => parse_with_lexer(ShellToken::lexer, lines, initial_state),
        "swift" => parse_with_lexer(SwiftToken::lexer, lines, initial_state),
        "toml" => parse_with_lexer(TomlToken::lexer, lines, initial_state),
        "typst" => parse_with_lexer(TypstToken::lexer, lines, initial_state),
        "zig" => parse_with_lexer(ZigToken::lexer, lines, initial_state),
        _ => return None,
    })
}

pub fn filetype_tokens(filetype: &str) -> Option<Vec<AvailableToken>> {
    Some(match filetype {
        "c" => c_tokens(),
        "clojure" => clojure_tokens(),
        "cpp" => cpp_tokens(),
        "csharp" => csharp_tokens(),
        "dart" => dart_tokens(),
        "elixir" => elixir_tokens(),
        "erlang" => erlang_tokens(),
        "fsharp" => fsharp_tokens(),
        "go" => go_tokens(),
        "haskell" => haskell_tokens(),
        "java" => java_tokens(),
        "javascript" | "typescript" | "javascriptreact" | "typescriptreact" => javascript_tokens(),
        "json" | "json5" | "jsonc" => json_tokens(),
        "kotlin" => kotlin_tokens(),
        "latex" => latex_tokens(),
        "lean" => lean_tokens(),
        "lua" => lua_tokens(),
        "objc" => objc_tokens(),
        "ocaml" => ocaml_tokens(),
        "perl" => perl_tokens(),
        "php" => php_tokens(),
        "python" => python_tokens(),
        "r" => r_tokens(),
        "ruby" => ruby_tokens(),
        "rust" => rust_tokens(),
        "scala" => scala_tokens(),
        "shell" => shell_tokens(),
        "swift" => swift_tokens(),
        "toml" => toml_tokens(),
        "typst" => typst_tokens(),
        "zig" => zig_tokens(),
        _ => return None,
    })
}

fn parse_with_lexer<'s, T>(
    mut lexer: impl FnMut(&'s str) -> Lexer<'s, T>,
    lines: &[&'s str],
    initial_state: ParseState,
) -> (Vec<Vec<Match>>, Vec<ParseState>)
where
    T: Into<Token> + Logos<'s>,
{
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut state_by_line = Vec::with_capacity(lines.len());
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
                (Normal, DelimiterOpen { text, closing }, false) => {
                    let match_ = Match {
                        type_: TokenType::Delimiter,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: Some(stack.len()),
                    };
                    stack.push(closing);
                    current_line_matches.push(match_);
                }
                (Normal, DelimiterClose(text), false) => {
                    if let Some(closing) = stack.last() {
                        if text == *closing {
                            stack.pop();
                        }
                    }

                    let match_ = Match {
                        type_: TokenType::Delimiter,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: Some(stack.len()),
                    };
                    current_line_matches.push(match_);
                }

                // Line comment - stop parsing rest of line
                (Normal, LineComment, false) => break,

                // Block comment
                (Normal, BlockCommentOpen { text, closing }, _) => {
                    let match_ = Match {
                        type_: TokenType::BlockComment,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                    };
                    current_line_matches.push(match_);
                    state = InBlockComment(closing)
                }
                (InBlockComment(closing), BlockCommentClose(text), _) if *closing == text => {
                    let match_ = Match {
                        type_: TokenType::BlockComment,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                    };
                    current_line_matches.push(match_);
                    state = Normal
                }

                // Block string
                (Normal, BlockStringOpen { text, closing }, _) => {
                    let match_ = Match {
                        type_: TokenType::String,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                    };
                    current_line_matches.push(match_);
                    state = InBlockString(closing)
                }
                (Normal, BlockStringSymmetric(text), _) => {
                    let match_ = Match {
                        type_: TokenType::String,
                        text,
                        col: lexer.span().start,
                        closing: Some(text),
                        stack_height: None,
                    };
                    current_line_matches.push(match_);
                    state = InBlockString(text)
                }
                (
                    InBlockString(closing),
                    BlockStringClose(text) | BlockStringSymmetric(text),
                    _,
                ) if *closing == text => {
                    let match_ = Match {
                        type_: TokenType::String,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                    };
                    current_line_matches.push(match_);
                    state = Normal
                }

                (_, Escape, false) => escaped_position = Some(lexer.span().end),
                _ => {}
            }
        }

        matches_by_line.push(current_line_matches);
        state_by_line.push(state.clone());
    }

    (matches_by_line, state_by_line)
}
