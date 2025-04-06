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

/// Given a matcher, runs the tokenizer on the lines and keeps track
/// of the state and matches for each line
pub fn parse<M: Matcher>(
    lines: &[&str],
    initial_state: State,
    mut matcher: M,
) -> (Vec<Vec<Match>>, Vec<State>)
where
    M: Matcher,
{
    // State
    let mut matches_by_line = Vec::with_capacity(lines.len());
    let mut line_matches = vec![];

    let mut state_by_line = Vec::with_capacity(lines.len());
    let mut state = initial_state;

    let mut stack = vec![];

    let text = lines.join("\n");
    let mut tokens = tokenize(&text, matcher.tokens()).multipeek();
    while let Some(token) = tokens.next() {
        // New line
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

// TODO: come up with a better way to do testing
#[cfg(test)]
mod tests {
    use crate::parser::{parse_filetype, Match, State};

    fn parse_c(lines: &str) -> Vec<Vec<Match>> {
        parse_filetype("c", &lines.split('\n').collect::<Vec<_>>(), State::Normal)
            .unwrap()
            .0
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse_c("{\n}"),
            vec![
                vec![Match::delimiter('{', 0, Some(0))],
                vec![Match::delimiter('}', 0, Some(0))]
            ]
        );

        assert_eq!(
            parse_c("// comment {}\n}"),
            vec![
                vec![Match::line_comment("//", 0)],
                vec![Match::delimiter('}', 0, Some(0))],
            ]
        );

        assert_eq!(
            parse_c("/* comment {} */\n}"),
            vec![
                vec![
                    Match::block_comment("/*", 0),
                    Match::block_comment("*/", 14)
                ],
                vec![Match::delimiter('}', 0, Some(0))]
            ]
        );
    }
}
