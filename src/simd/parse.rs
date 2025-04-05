use itertools::Itertools;

use super::{
    matcher::{Match, Matcher},
    tokenize::tokenize,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Normal,
    InString(&'static str),
    InBlockString(&'static str),
    InLineComment,
    InBlockComment(&'static str),
}

pub fn parse<M: Matcher>(
    lines: &[&str],
    initial_state: State,
    mut matcher: M,
) -> (Vec<Vec<Match>>, Vec<State>)
where
    M: Matcher,
{
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut line_matches = vec![];

    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut state = initial_state;

    let mut stack = vec![];

    let text = lines.join("\n");

    let mut tokens = tokenize(&text, matcher.tokens()).multipeek();
    while let Some(token) = tokens.next() {
        if matches!(token.byte, b'\n') {
            matches_by_line.push(line_matches);
            line_matches = vec![];

            if matches!(state, State::InString(_) | State::InLineComment) {
                state = State::Normal;
            }
            state_by_line.push(state);
            continue;
        }

        state = matcher.call(&mut line_matches, &mut stack, &mut tokens, state, token);
    }
    matches_by_line.push(line_matches);
    state_by_line.push(state);

    (matches_by_line, state_by_line)
}

#[cfg(test)]
mod tests {
    use crate::simd::parse_language;

    use super::*;
    use crate::simd::MatchToken;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse_language("c", &["{", "}"], State::Normal).unwrap().0,
            vec![
                vec![Match {
                    token: MatchToken::DelimiterOpen("{", "}"),
                    col: 0,
                    stack_height: Some(0)
                }],
                vec![Match {
                    token: MatchToken::DelimiterClose("{", "}"),
                    col: 0,
                    stack_height: Some(0)
                }]
            ]
        );

        assert_eq!(
            parse_language("c", &["// comment {}", "}"], State::Normal)
                .unwrap()
                .0,
            vec![
                vec![Match::new(MatchToken::LineComment("//"), 0)],
                vec![Match {
                    token: MatchToken::DelimiterClose("{", "}"),
                    col: 0,
                    stack_height: Some(0)
                }],
            ]
        );

        assert_eq!(
            parse_language("c", &["/* comment {} */", "}"], State::Normal)
                .unwrap()
                .0,
            vec![
                vec![
                    Match::new(MatchToken::BlockCommentOpen("/*", "*/"), 0),
                    Match::new(MatchToken::BlockCommentClose("/*", "*/"), 14)
                ],
                vec![Match {
                    token: MatchToken::DelimiterClose("{", "}"),
                    col: 0,
                    stack_height: Some(0)
                }],
            ]
        );
    }
}
