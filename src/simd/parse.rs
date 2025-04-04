use super::tokenize::{tokenize, SimdToken, SimdTokenType};

#[derive(Debug, PartialEq)]
pub struct SimdMatch {
    pub token: SimdMatchType,
    pub col: usize,
}

impl From<SimdToken> for SimdMatch {
    fn from(token: SimdToken) -> Self {
        SimdMatch {
            token: token.token.into(),
            col: token.col,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SimdMatchType {
    DelimiterOpen(&'static str, &'static str),
    DelimiterClose(&'static str, &'static str),

    StringOpen(&'static str),
    StringClose(&'static str),
    BlockStringOpen(&'static str, &'static str),
    BlockStringClose(&'static str, &'static str),

    LineComment(&'static str),
    BlockCommentOpen(&'static str, &'static str),
    BlockCommentClose(&'static str, &'static str),
}

impl From<SimdTokenType> for SimdMatchType {
    fn from(token: SimdTokenType) -> Self {
        match token {
            SimdTokenType::CurlyBraceOpen => SimdMatchType::DelimiterOpen("{", "}"),
            SimdTokenType::CurlyBraceClose => SimdMatchType::DelimiterClose("{", "}"),
            SimdTokenType::SquareBracketOpen => SimdMatchType::DelimiterOpen("[", "]"),
            SimdTokenType::SquareBracketClose => SimdMatchType::DelimiterClose("[", "]"),
            SimdTokenType::ParenthesisOpen => SimdMatchType::DelimiterOpen("(", ")"),
            SimdTokenType::ParenthesisClose => SimdMatchType::DelimiterClose("(", ")"),

            _ => panic!("Invalid or ambiguous token type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Normal,
    InString(&'static str),
    InBlockString(&'static str),
    InLineComment,
    InBlockComment(&'static str),
}

pub fn parse(lines: &[&str]) -> (Vec<Vec<SimdMatch>>, Vec<State>) {
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut line_matches = vec![];

    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut state = State::Normal;

    let text = lines.join("\n");

    let mut tokens = tokenize(&text).peekable();
    while let Some(token) = tokens.next() {
        use super::tokenize::SimdTokenType::*;
        use State::*;

        let col = token.col;

        match (
            state,
            token.token,
            tokens.peek().map(|t| t.token).unwrap_or(None),
        ) {
            // New line
            (_, NewLine, _) => {
                matches_by_line.push(line_matches);
                line_matches = vec![];

                if matches!(state, InString(_) | InLineComment) {
                    state = Normal;
                }
                state_by_line.push(state);
            }

            // Delimiters
            (
                Normal,
                CurlyBraceOpen | CurlyBraceClose | SquareBracketOpen | SquareBracketClose
                | ParenthesisOpen | ParenthesisClose,
                _,
            ) => line_matches.push(token.into()),

            // Strings
            (Normal, SingleQuote, _) => {
                state = InString("'");
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringOpen("'"),
                    col,
                });
            }
            (InString(delim), SingleQuote, _) if delim == "'" => {
                state = Normal;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringClose(delim),
                    col,
                });
            }

            (Normal, DoubleQuote, _) => {
                state = InString("\"");
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringOpen("\""),
                    col,
                });
            }
            (InString(delim), DoubleQuote, _) if delim == "\"" => {
                state = Normal;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringClose(delim),
                    col,
                });
            }

            // Line comments
            (Normal, ForwardSlash, ForwardSlash) => {
                state = InLineComment;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::LineComment("//"),
                    col,
                });
                tokens.next(); // Skip next token
            }

            // Block comments
            (Normal, ForwardSlash, Star) => {
                state = InBlockComment("/*");
                line_matches.push(SimdMatch {
                    token: SimdMatchType::BlockCommentOpen("/*", "*/"),
                    col,
                });
                tokens.next(); // Skip next token
            }
            (InBlockComment("/*"), Star, ForwardSlash) => {
                state = Normal;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::BlockCommentClose("/*", "*/"),
                    col,
                });
                tokens.next(); // Skip next token
            }

            _ => {}
        }
    }
    matches_by_line.push(line_matches);
    state_by_line.push(state);

    (matches_by_line, state_by_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse(&["{", "}"]).0,
            vec![
                vec![SimdMatch {
                    token: SimdMatchType::DelimiterOpen("{", "}"),
                    col: 0,
                }],
                vec![SimdMatch {
                    token: SimdMatchType::DelimiterClose("{", "}"),
                    col: 0,
                }]
            ]
        );

        assert_eq!(
            parse(&["// comment {}", "}"]).0,
            vec![
                vec![SimdMatch {
                    token: SimdMatchType::LineComment("//"),
                    col: 0
                }],
                vec![SimdMatch {
                    token: SimdMatchType::DelimiterClose("{", "}"),
                    col: 0
                }]
            ]
        );

        assert_eq!(
            parse(&["/* comment {} */", "}"]).0,
            vec![
                vec![
                    SimdMatch {
                        token: SimdMatchType::BlockCommentOpen("/*", "*/"),
                        col: 0
                    },
                    SimdMatch {
                        token: SimdMatchType::BlockCommentClose("/*", "*/"),
                        col: 14
                    }
                ],
                vec![SimdMatch {
                    token: SimdMatchType::DelimiterClose("{", "}"),
                    col: 0
                }]
            ]
        );
    }
}
