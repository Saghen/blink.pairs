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
    InInlineSpan(&'static str),
    InBlockSpan(&'static str),
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
    let mut escaped_col: Option<usize> = None;

    let text = lines.join("\n");

    #[cfg(target_feature = "avx512f")]
    let tokens = tokenize::<64>(&text, matcher.tokens());
    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    let tokens = tokenize::<32>(&text, matcher.tokens());
    #[cfg(not(any(target_feature = "avx2", target_feature = "avx512f")))]
    let mut tokens = tokenize::<16>(&text, matcher.tokens());

    let mut tokens = tokens.multipeek();

    while let Some(token) = tokens.next() {
        // New line
        if matches!(token.byte, b'\n') {
            matches_by_line.push(line_matches);
            line_matches = vec![];
            escaped_col = None;

            if matches!(
                state,
                State::InString(_) | State::InLineComment | State::InInlineSpan(_)
            ) {
                state = State::Normal;
            }
            state_by_line.push(state);
            continue;
        }

        if matches!(token.byte, b'\\') {
            if let Some(col) = escaped_col {
                if col == token.col - 1 {
                    escaped_col = None;
                    continue;
                }
            }
            escaped_col = Some(token.col);
            continue;
        }

        state = matcher.call(
            &mut line_matches,
            &mut stack,
            &mut tokens,
            state,
            token,
            escaped_col.map(|col| col == token.col - 1).unwrap_or(false),
        );
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
