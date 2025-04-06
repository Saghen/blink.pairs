use crate::parser::{parse_filetype, Kind, Match, MatchWithLine, State};

pub struct ParsedBuffer {
    matches_by_line: Vec<Vec<Match>>,
    state_by_line: Vec<State>,
}

impl ParsedBuffer {
    pub fn parse(filetype: &str, lines: &[&str]) -> Option<Self> {
        let (matches_by_line, state_by_line) = parse_filetype(filetype, lines, State::Normal)?;

        Some(Self {
            matches_by_line,
            state_by_line,
        })
    }

    pub fn reparse_range(
        &mut self,
        filetype: &str,
        lines: &[&str],
        start_line: Option<usize>,
        old_end_line: Option<usize>,
        new_end_line: Option<usize>,
    ) -> bool {
        let max_line = self.matches_by_line.len();
        let start_line = start_line.unwrap_or(0).min(max_line);
        let old_end_line = old_end_line.unwrap_or(max_line).min(max_line);

        let initial_state = if start_line > 0 {
            self.state_by_line
                .get(start_line - 1)
                .cloned()
                .unwrap_or(State::Normal)
        } else {
            State::Normal
        };

        if let Some((matches_by_line, state_by_line)) =
            parse_filetype(filetype, lines, initial_state)
        {
            let new_end_line = new_end_line.unwrap_or(start_line + matches_by_line.len());
            let length = new_end_line - start_line;
            self.matches_by_line.splice(
                start_line..old_end_line,
                matches_by_line[0..length].to_vec(),
            );
            self.state_by_line
                .splice(start_line..old_end_line, state_by_line[0..length].to_vec());

            self.recalculate_stack_heights();

            true
        } else {
            false
        }
    }

    pub fn line_matches(&self, line_number: usize) -> Option<Vec<Match>> {
        self.matches_by_line.get(line_number).cloned()
    }

    pub fn match_at(&self, line_number: usize, col: usize) -> Option<Match> {
        self.matches_by_line
            .get(line_number)?
            .iter()
            .find(|match_| (match_.col..(match_.col + match_.len())).contains(&col))
            .cloned()
    }

    pub fn match_pair(
        &self,
        line_number: usize,
        col: usize,
    ) -> Option<(MatchWithLine, MatchWithLine)> {
        let match_at_pos = self.match_at(line_number, col)?.with_line(line_number);

        // Opening match
        if match_at_pos.kind == Kind::Opening {
            let closing_match = self.matches_by_line[line_number..]
                .iter()
                .enumerate()
                .map(|(matches_line_number, matches)| (matches_line_number + line_number, matches))
                .find_map(|(matches_line_number, matches)| {
                    matches
                        .iter()
                        .find(|match_| {
                            (line_number != matches_line_number || match_.col > col)
                                && match_at_pos.token == match_.token
                                && match_at_pos.stack_height == match_.stack_height
                        })
                        .map(|match_| match_.with_line(matches_line_number))
                })?;

            Some((match_at_pos, closing_match))
        }
        // Closing match
        else if match_at_pos.kind == Kind::Closing {
            let opening_match = self.matches_by_line[0..=line_number]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(matches_line_number, matches)| {
                    matches
                        .iter()
                        .rev()
                        .find(|match_| {
                            (line_number != matches_line_number || match_.col + match_.len() < col)
                                && match_at_pos.token == match_.token
                                && match_at_pos.stack_height == match_.stack_height
                        })
                        .map(|match_| match_.with_line(matches_line_number))
                })?;

            Some((opening_match, match_at_pos))
        } else {
            None
        }
    }

    fn recalculate_stack_heights(&mut self) {
        let mut stack = vec![];

        for matches in self.matches_by_line.iter_mut() {
            for match_ in matches {
                // Opening delimiter
                if match_.kind == Kind::Opening {
                    match_.stack_height = Some(stack.len());
                    stack.push(&match_.token);
                }
                // Closing delimiter
                else {
                    if let Some(closing) = stack.last() {
                        if *closing == &match_.token {
                            stack.pop();
                        }
                    }
                    match_.stack_height = Some(stack.len());
                }
            }
        }
    }
}
