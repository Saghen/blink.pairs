use itertools::{Itertools, MultiPeek};

use super::tokenize::{tokenize, SimdToken, SimdTokenType};

#[derive(Debug, PartialEq)]
pub struct SimdMatch {
    pub token: SimdMatchType,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl SimdMatch {
    pub fn new(token: SimdMatchType, col: usize) -> Self {
        Self {
            token,
            col,
            stack_height: None,
        }
    }
}

impl From<SimdToken> for SimdMatch {
    fn from(token: SimdToken) -> Self {
        SimdMatch {
            token: token.token.into(),
            col: token.col,
            stack_height: None,
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

pub fn parse(
    lines: &[&str],
    initial_state: State,
    matcher: fn(
        &mut Vec<SimdMatch>,
        &mut Vec<SimdTokenType>,
        &mut MultiPeek<impl Iterator<Item = SimdToken>>,
        State,
        SimdToken,
    ) -> State,
) -> (Vec<Vec<SimdMatch>>, Vec<State>) {
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut line_matches = vec![];

    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut state = initial_state;

    let mut stack = vec![];

    let text = lines.join("\n");

    let mut tokens = tokenize(&text).multipeek();
    while let Some(token) = tokens.next() {
        if matches!(token.token, SimdTokenType::NewLine) {
            matches_by_line.push(line_matches);
            line_matches = vec![];

            if matches!(state, State::InString(_) | State::InLineComment) {
                state = State::Normal;
            }
            state_by_line.push(state);
            continue;
        }

        matcher(&mut line_matches, &mut stack, &mut tokens, state, token);
    }
    matches_by_line.push(line_matches);
    state_by_line.push(state);

    (matches_by_line, state_by_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simd::languages::c_matcher;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse(&["{", "}"], State::Normal, c_matcher).0,
            vec![
                vec![SimdMatch::new(SimdMatchType::DelimiterOpen("{", "}"), 0)],
                vec![SimdMatch::new(SimdMatchType::DelimiterClose("{", "}"), 0)]
            ]
        );

        assert_eq!(
            parse(&["// comment {}", "}"], State::Normal, c_matcher).0,
            vec![
                vec![SimdMatch::new(SimdMatchType::LineComment("//"), 0)],
                vec![SimdMatch::new(SimdMatchType::DelimiterClose("{", "}"), 0)]
            ]
        );

        assert_eq!(
            parse(&["/* comment {} */", "}"], State::Normal, c_matcher).0,
            vec![
                vec![
                    SimdMatch::new(SimdMatchType::BlockCommentOpen("/*", "*/"), 0),
                    SimdMatch::new(SimdMatchType::BlockCommentClose("/*", "*/"), 14)
                ],
                vec![SimdMatch::new(SimdMatchType::DelimiterClose("{", "}"), 0)]
            ]
        );
    }
}
