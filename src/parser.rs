use logos::{Lexer, Logos};
use mlua::{serde::Serializer, IntoLua, SerializeOptions};
use serde::Serialize;
use std::collections::HashMap;

use super::languages::*;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    #[serde(skip)]
    pub type_: TokenType,

    pub text: &'static str,
    pub closing: Option<&'static str>,

    pub col: usize,
    pub stack_height: Option<usize>,
    pub span_name: Option<&'static str>,
    pub span_id: Option<usize>,
}

impl Match {
    pub fn with_line(&self, line: usize) -> MatchWithLine {
        MatchWithLine {
            type_: self.type_,
            text: self.text,
            col: self.col,
            closing: self.closing,
            stack_height: self.stack_height,
            span_name: self.span_name,
            span_id: self.span_id,
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
    pub span_name: Option<&'static str>,
    pub span_id: Option<usize>,
}

impl IntoLua for MatchWithLine {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new_with_options(
            lua,
            SerializeOptions::new().serialize_none_to_null(false),
        ))
    }
}

/// A structured representation of a span in the document
#[derive(Debug, Clone)]
pub struct Span {
    /// Unique identifier for the span
    pub id: usize,
    /// The type of span (inline or block)
    pub type_: TokenType,
    /// Semantic name of the span
    pub name: &'static str,
    /// Opening token information
    pub opening: SpanToken,
    /// Closing token information (None if unclosed)
    pub closing: Option<SpanToken>,
    /// Parent span ID if this span is nested
    pub parent_id: Option<usize>,
    /// Child span IDs
    pub children: Vec<usize>,
}

/// A token that marks the beginning or end of a span
#[derive(Debug, Clone)]
pub struct SpanToken {
    /// Line number where the token appears
    pub line: usize,
    /// Column position where the token appears
    pub col: usize,
    /// The actual text of the token
    pub text: &'static str,
}

#[derive(Debug, Clone)]
pub enum ParseState {
    Normal,
    InBlockComment(&'static str),
    InBlockString(&'static str),
    InInlineSpan {
        closing: &'static str,
        name: &'static str,
        span_id: usize,
    },
    InBlockSpan {
        closing: &'static str,
        name: &'static str,
        span_id: usize,
    },
}

pub fn parse_filetype(
    filetype: &str,
    lines: &[&str],
    initial_state: ParseState,
) -> Option<(Vec<Vec<Match>>, Vec<ParseState>, Vec<Span>)> {
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
        "markdown" | "md" => parse_with_lexer(MarkdownToken::lexer, lines, initial_state),
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
        "markdown" | "md" => markdown_tokens(),
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
) -> (Vec<Vec<Match>>, Vec<ParseState>, Vec<Span>)
where
    T: Into<Token> + Logos<'s>,
{
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut stack = vec![];

    let mut spans = Vec::new();
    let mut next_span_id = 0;
    let mut span_stack = Vec::new();
    let mut active_spans: HashMap<(TokenType, &'static str), Vec<usize>> = HashMap::new();
    let mut parent_stack: Vec<usize> = Vec::new();

    let mut state = initial_state;
    for (line_idx, line) in lines.iter().enumerate() {
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
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
                        span_name: None,
                        span_id: None,
                    };
                    current_line_matches.push(match_);
                    state = Normal
                }

                // Inline spans
                (
                    Normal,
                    InlineSpanOpen {
                        text,
                        closing,
                        name,
                    },
                    _,
                ) => {
                    let span_id = next_span_id;
                    next_span_id += 1;

                    let span = Span {
                        id: span_id,
                        type_: TokenType::InlineSpan,
                        name,
                        opening: SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        },
                        closing: None,
                        parent_id: parent_stack.last().copied(),
                        children: Vec::new(),
                    };
                    spans.push(span);

                    active_spans
                        .entry((TokenType::InlineSpan, name))
                        .or_default()
                        .push(span_id);

                    span_stack.push(span_id);
                    parent_stack.push(span_id);

                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                        span_name: Some(name),
                        span_id: Some(span_id),
                    };
                    current_line_matches.push(match_);
                    state = InInlineSpan {
                        closing,
                        name,
                        span_id,
                    };
                }
                (Normal, InlineSpanSymmetric { text, name }, _) => {
                    let span_id = next_span_id;
                    next_span_id += 1;

                    let span = Span {
                        id: span_id,
                        type_: TokenType::InlineSpan,
                        name,
                        opening: SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        },
                        closing: None,
                        parent_id: parent_stack.last().copied(),
                        children: Vec::new(),
                    };
                    spans.push(span);

                    active_spans
                        .entry((TokenType::InlineSpan, name))
                        .or_default()
                        .push(span_id);

                    span_stack.push(span_id);
                    parent_stack.push(span_id);

                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(text),
                        stack_height: None,
                        span_name: Some(name),
                        span_id: Some(span_id),
                    };
                    current_line_matches.push(match_);
                    state = InInlineSpan {
                        closing: text,
                        name,
                        span_id,
                    };
                }
                (
                    InInlineSpan {
                        closing,
                        name,
                        span_id,
                    },
                    InlineSpanClose(text) | InlineSpanSymmetric { text, .. },
                    _,
                ) if *closing == text => {
                    let current_span_id = *span_id;

                    if let Some(span) = spans.iter_mut().find(|s| s.id == current_span_id) {
                        span.closing = Some(SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        });
                    }

                    if let Some(active) = active_spans.get_mut(&(TokenType::InlineSpan, *name)) {
                        if let Some(pos) = active.iter().position(|id| *id == current_span_id) {
                            active.remove(pos);
                        }
                    }

                    if let Some(pos) = span_stack.iter().position(|id| *id == current_span_id) {
                        span_stack.remove(pos);
                    }

                    if parent_stack.last() == Some(&current_span_id) {
                        parent_stack.pop();
                    }

                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                        span_name: Some(*name),
                        span_id: Some(current_span_id),
                    };
                    current_line_matches.push(match_);
                    state = Normal;
                }

                // Block spans
                (
                    Normal,
                    BlockSpanOpen {
                        text,
                        closing,
                        name,
                    },
                    _,
                ) => {
                    let span_id = next_span_id;
                    next_span_id += 1;

                    let span = Span {
                        id: span_id,
                        type_: TokenType::BlockSpan,
                        name,
                        opening: SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        },
                        closing: None,
                        parent_id: parent_stack.last().copied(),
                        children: Vec::new(),
                    };
                    spans.push(span);

                    active_spans
                        .entry((TokenType::BlockSpan, name))
                        .or_default()
                        .push(span_id);

                    span_stack.push(span_id);
                    parent_stack.push(span_id);

                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                        span_name: Some(name),
                        span_id: Some(span_id),
                    };
                    current_line_matches.push(match_);
                    state = InBlockSpan {
                        closing,
                        name,
                        span_id,
                    };
                }
                (Normal, BlockSpanSymmetric { text, name }, _) => {
                    let span_id = next_span_id;
                    next_span_id += 1;

                    let span = Span {
                        id: span_id,
                        type_: TokenType::BlockSpan,
                        name,
                        opening: SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        },
                        closing: None,
                        parent_id: parent_stack.last().copied(),
                        children: Vec::new(),
                    };
                    spans.push(span);

                    active_spans
                        .entry((TokenType::BlockSpan, name))
                        .or_default()
                        .push(span_id);

                    span_stack.push(span_id);
                    parent_stack.push(span_id);

                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(text),
                        stack_height: None,
                        span_name: Some(name),
                        span_id: Some(span_id),
                    };
                    current_line_matches.push(match_);
                    state = InBlockSpan {
                        closing: text,
                        name,
                        span_id,
                    };
                }
                (
                    InBlockSpan {
                        closing,
                        name,
                        span_id,
                    },
                    BlockSpanClose(text) | BlockSpanSymmetric { text, .. },
                    _,
                ) if *closing == text => {
                    let current_span_id = *span_id;

                    if let Some(span) = spans.iter_mut().find(|s| s.id == current_span_id) {
                        span.closing = Some(SpanToken {
                            line: line_idx,
                            col: lexer.span().start,
                            text,
                        });
                    }

                    if let Some(active) = active_spans.get_mut(&(TokenType::BlockSpan, *name)) {
                        if let Some(pos) = active.iter().position(|id| *id == current_span_id) {
                            active.remove(pos);
                        }
                    }

                    if let Some(pos) = span_stack.iter().position(|id| *id == current_span_id) {
                        span_stack.remove(pos);
                    }

                    if parent_stack.last() == Some(&current_span_id) {
                        parent_stack.pop();
                    }

                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                        span_name: Some(*name),
                        span_id: Some(current_span_id),
                    };
                    current_line_matches.push(match_);
                    state = Normal;
                }

                (_, Escape, false) => escaped_position = Some(lexer.span().end),
                _ => {}
            }
        }

        matches_by_line.push(current_line_matches);
        state_by_line.push(state.clone());
    }

    for i in 0..spans.len() {
        if let Some(parent_id) = spans[i].parent_id {
            let child_id = spans[i].id;

            if let Some(parent) = spans.iter_mut().find(|s| s.id == parent_id) {
                parent.children.push(child_id);
            }
        }
    }

    for span in spans.iter().filter(|s| s.closing.is_some()) {
        let start_line = span.opening.line;
        let end_line = span.closing.as_ref().unwrap().line;

        if span.type_ == TokenType::InlineSpan && start_line == end_line {
            let line = start_line;
            let start_col = span.opening.col + span.opening.text.len();
            let end_col = span.closing.as_ref().unwrap().col;

            if start_col >= end_col || start_col >= matches_by_line[line].len() {
                continue;
            }

            matches_by_line[line].push(Match {
                type_: span.type_,
                text: "",
                col: start_col,
                closing: None,
                stack_height: None,
                span_name: Some(span.name),
                span_id: Some(span.id),
            });
        } else {
            for line in start_line..=end_line {
                if line >= matches_by_line.len() {
                    continue;
                }

                if (line == start_line
                    && matches_by_line[line]
                        .iter()
                        .any(|m| m.span_id == Some(span.id) && m.closing.is_some()))
                    || (line == end_line
                        && matches_by_line[line]
                            .iter()
                            .any(|m| m.span_id == Some(span.id) && m.closing.is_none()))
                {
                    continue;
                }

                matches_by_line[line].push(Match {
                    type_: span.type_,
                    text: "",
                    col: 0,
                    closing: None,
                    stack_height: None,
                    span_name: Some(span.name),
                    span_id: Some(span.id),
                });
            }
        }
    }

    (matches_by_line, state_by_line, spans)
}
