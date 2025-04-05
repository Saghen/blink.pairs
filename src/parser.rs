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
    pub span_name: Option<&'static str>,
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
    InInlineSpan {
        closing: &'static str,
        name: &'static str,
    },
    InBlockSpan {
        closing: &'static str,
        name: &'static str,
    },
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
) -> (Vec<Vec<Match>>, Vec<ParseState>)
where
    T: Into<Token> + Logos<'s>,
{
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut stack = vec![];
    
    #[derive(Debug)]
    struct SpanToken {
        type_: TokenType, 
        name: &'static str,
        opening_line: usize,
        opening_col: usize,
        closing_line: Option<usize>,
        closing_col: Option<usize>,
    }
    let mut span_stack: Vec<SpanToken> = vec![];

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
                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                        span_name: Some(name),
                    };
                    current_line_matches.push(match_);
                    state = InInlineSpan { closing, name };
                    
                    span_stack.push(SpanToken {
                        type_: TokenType::InlineSpan,
                        name,
                        opening_line: line_idx,
                        opening_col: lexer.span().start,
                        closing_line: None,
                        closing_col: None,
                    });
                }
                (Normal, InlineSpanSymmetric { text, name }, _) => {
                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(text),
                        stack_height: None,
                        span_name: Some(name),
                    };
                    current_line_matches.push(match_);
                    state = InInlineSpan {
                        closing: text,
                        name,
                    };
                    
                    span_stack.push(SpanToken {
                        type_: TokenType::InlineSpan,
                        name,
                        opening_line: line_idx,
                        opening_col: lexer.span().start,
                        closing_line: None,
                        closing_col: None,
                    });
                }
                (
                    InInlineSpan { closing, name },
                    InlineSpanClose(text) | InlineSpanSymmetric { text, .. },
                    _,
                ) if *closing == text => {
                    let current_name = *name;

                    let match_ = Match {
                        type_: TokenType::InlineSpan,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                        span_name: Some(current_name),
                    };
                    current_line_matches.push(match_);
                    state = Normal;
                    
                    for i in (0..span_stack.len()).rev() {
                        let span = &mut span_stack[i];
                        if span.type_ == TokenType::InlineSpan 
                           && span.name == current_name 
                           && span.closing_line.is_none() {
                            span.closing_line = Some(line_idx);
                            span.closing_col = Some(lexer.span().start);
                            break;
                        }
                    }
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
                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(closing),
                        stack_height: None,
                        span_name: Some(name),
                    };
                    current_line_matches.push(match_);
                    state = InBlockSpan { closing, name };
                    
                    span_stack.push(SpanToken {
                        type_: TokenType::BlockSpan,
                        name,
                        opening_line: line_idx,
                        opening_col: lexer.span().start,
                        closing_line: None,
                        closing_col: None,
                    });
                }
                (Normal, BlockSpanSymmetric { text, name }, _) => {
                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: Some(text),
                        stack_height: None,
                        span_name: Some(name),
                    };
                    current_line_matches.push(match_);
                    state = InBlockSpan {
                        closing: text,
                        name,
                    };
                    
                    span_stack.push(SpanToken {
                        type_: TokenType::BlockSpan,
                        name,
                        opening_line: line_idx,
                        opening_col: lexer.span().start,
                        closing_line: None,
                        closing_col: None,
                    });
                }
                (
                    InBlockSpan { closing, name },
                    BlockSpanClose(text) | BlockSpanSymmetric { text, .. },
                    _,
                ) if *closing == text => {
                    let current_name = *name;

                    let match_ = Match {
                        type_: TokenType::BlockSpan,
                        text,
                        col: lexer.span().start,
                        closing: None,
                        stack_height: None,
                        span_name: Some(current_name),
                    };
                    current_line_matches.push(match_);
                    state = Normal;
                    
                    for i in (0..span_stack.len()).rev() {
                        let span = &mut span_stack[i];
                        if span.type_ == TokenType::BlockSpan 
                           && span.name == current_name 
                           && span.closing_line.is_none() {
                            span.closing_line = Some(line_idx);
                            span.closing_col = Some(lexer.span().start);
                            break;
                        }
                    }
                }

                (_, Escape, false) => escaped_position = Some(lexer.span().end),
                _ => {}
            }
        }

        matches_by_line.push(current_line_matches);
        state_by_line.push(state.clone());
    }
    
    // Post-process: mark valid span regions
    for span in span_stack.iter().filter(|span| span.closing_line.is_some()) {
        let start_line = span.opening_line;
        let end_line = span.closing_line.unwrap();
        
        // For inline spans, only mark characters between opening and closing on the same line
        if span.type_ == TokenType::InlineSpan && start_line == end_line {
            let line = start_line;
            
            let start_col;
            if let Some(opening_match) = matches_by_line[line].iter().find(|m| 
                m.col == span.opening_col && m.closing.is_some() && m.span_name == Some(span.name)
            ) {
                start_col = span.opening_col + opening_match.text.len();
            } else {
                start_col = span.opening_col + 1;
            }
            
            let end_col = span.closing_col.unwrap();
            
            for col in start_col..end_col {
                let already_marked = matches_by_line[line].iter().any(|m| m.col == col);
                if !already_marked {
                    matches_by_line[line].push(Match {
                        type_: span.type_,
                        text: "",
                        col,
                        closing: None,
                        stack_height: None,
                        span_name: Some(span.name),
                    });
                }
            }
        } 
        // For block spans or multi-line inline spans, mark everything between start and end lines
        else {
            // Update all state lines to reflect the block span state
            for line in start_line..end_line {
                if line >= state_by_line.len() {
                    continue;
                }
                
                if let ParseState::Normal = state_by_line[line] {
                    state_by_line[line] = ParseState::InBlockSpan {
                        closing: if let Some(opening_match) = matches_by_line[start_line].iter().find(|m| 
                            m.col == span.opening_col && m.closing.is_some() && m.span_name == Some(span.name)
                        ) {
                            if let Some(close) = opening_match.closing {
                                close
                            } else {
                                ""
                            }
                        } else {
                            ""
                        },
                        name: span.name,
                    };
                }
            }
            
            // Now mark all positions inside the span
            for line in start_line..=end_line {
                if line >= lines.len() || line >= matches_by_line.len() {
                    continue;
                }
                
                let (start_col, end_col) = match line {
                    l if l == start_line => {
                        let start_col;
                        if let Some(opening_match) = matches_by_line[line].iter().find(|m| 
                            m.col == span.opening_col && m.closing.is_some() && m.span_name == Some(span.name)
                        ) {
                            start_col = span.opening_col + opening_match.text.len();
                        } else {
                            start_col = span.opening_col + 1;
                        }
                        (start_col, lines[line].len())
                    },
                    l if l == end_line => (0, span.closing_col.unwrap()),
                    _ => (0, lines[line].len()),
                };
                
                // Mark entire line if it's a middle line in a block span
                if span.type_ == TokenType::BlockSpan && line > start_line && line < end_line {
                    matches_by_line[line].push(Match {
                        type_: span.type_,
                        text: "",
                        col: 0,
                        closing: None,
                        stack_height: None,
                        span_name: Some(span.name),
                    });
                    continue;
                }
                
                // Don't process if the indices are invalid
                if start_col >= end_col || (lines[line].len() > 0 && start_col >= lines[line].len()) {
                    continue;
                }
                
                // For actual content, mark each position
                let max_col = if lines[line].is_empty() { 1 } else { end_col };
                for col in start_col..max_col {
                    let already_marked = matches_by_line[line].iter().any(|m| m.col == col);
                    if !already_marked {
                        matches_by_line[line].push(Match {
                            type_: span.type_,
                            text: "",
                            col,
                            closing: None,
                            stack_height: None,
                            span_name: Some(span.name),
                        });
                    }
                }
                
                // For empty lines in block spans, ensure we add at least one marker
                if lines[line].is_empty() && !matches_by_line[line].iter().any(|m| m.span_name == Some(span.name)) {
                    matches_by_line[line].push(Match {
                        type_: span.type_,
                        text: "",
                        col: 0,
                        closing: None,
                        stack_height: None,
                        span_name: Some(span.name),
                    });
                }
            }
        }
    }

    // Also need to handle unclosed spans based on parser state
    // For any line that ends with a block span state, mark the next line as in that span
    let mut line_idx = 0;
    while line_idx < state_by_line.len() {
        if let ParseState::InBlockSpan { name, .. } = &state_by_line[line_idx] {
            // If we're in a block span at end of a line, but there's no closing token,
            // then we need to ensure the span continues
            let has_closing_on_line = matches_by_line[line_idx].iter().any(|m| m.span_name == Some(*name) && m.closing.is_none());
            if !has_closing_on_line && line_idx + 1 < matches_by_line.len() {
                // Add a span marker to the next line if needed
                let next_line = line_idx + 1;
                let has_span_marker = matches_by_line[next_line].iter().any(|m| m.span_name == Some(*name));
                if !has_span_marker {
                    matches_by_line[next_line].push(Match {
                        type_: TokenType::BlockSpan,
                        text: "",
                        col: 0,
                        closing: None,
                        stack_height: None,
                        span_name: Some(*name),
                    });
                }
            }
        }
        line_idx += 1;
    }

    (matches_by_line, state_by_line)
}
