use super::tokenize::{self, tokenize, SimdToken, SimdTokenType};

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

pub enum SimdMatchType {
    DelimiterOpen(&'static str, &'static str),
    DelimiterClose(&'static str, &'static str),

    StringOpen(&'static str),
    StringClose(&'static str),
    BlockStringOpen(&'static str),
    BlockStringClose(&'static str),

    LineComment(&'static str),
    BlockCommentOpen(&'static str),
    BlockCommentClose(&'static str),
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

pub fn parse(lines: &[&str]) -> Vec<Vec<SimdMatch>> {
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut line_matches = vec![];

    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut state = State::Normal;

    for token in tokenize(&lines.join("\n")) {
        use super::tokenize::SimdTokenType::*;
        use State::*;

        let col = token.col;
        match (state, token.token) {
            (_, NewLine) => {
                matches_by_line.push(line_matches);
                line_matches = vec![];

                if matches!(state, InString(_) | InLineComment) {
                    state = Normal;
                }
                state_by_line.push(state);
            }

            (
                Normal,
                CurlyBraceOpen | CurlyBraceClose | SquareBracketOpen | SquareBracketClose
                | ParenthesisOpen | ParenthesisClose,
            ) => line_matches.push(token.into()),

            (Normal, SingleQuote) => {
                state = InString("'");
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringOpen("'"),
                    col,
                });
            }
            (InString(delim), SingleQuote) if delim == "'" => {
                state = Normal;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringClose(delim),
                    col,
                });
            }

            (Normal, DoubleQuote) => {
                state = InString("\"");
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringOpen("\""),
                    col,
                });
            }
            (InString(delim), DoubleQuote) if delim == "\"" => {
                state = Normal;
                line_matches.push(SimdMatch {
                    token: SimdMatchType::StringClose(delim),
                    col,
                });
            }

            _ => {}
        }
    }
    matches_by_line.push(line_matches);
    state_by_line.push(state);

    matches_by_line
}
