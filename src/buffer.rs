use crate::parser::{parse_filetype, Kind, Match, MatchWithLine, State, Token};

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

    fn recalculate_stack_heights(&mut self) {
        let mut stack = vec![];

        // TODO: prefer matching on the furthest pair for mismatched openings
        // [ ( ( (  ) ]
        // ^ ^      ^ ^
        // Continue to match on closest pair for mismatched closings
        // [ ( ) ) ) ]
        // ^ ^ ^     ^
        for matches in self.matches_by_line.iter_mut() {
            'outer: for match_ in matches.iter_mut() {
                // Opening delimiter
                if match_.kind == Kind::Opening {
                    stack.push(match_);
                }
                // Closing delimiter
                else {
                    for (i, opening) in stack.iter().enumerate().rev() {
                        if opening.token == match_.token {
                            // Mark all skipped matches as unmatched
                            for unmatched_opening in stack.splice((i + 1).., vec![]) {
                                unmatched_opening.stack_height = None;
                            }

                            // Update stack height
                            let opening = stack.pop().unwrap();
                            opening.stack_height = Some(stack.len());
                            match_.stack_height = Some(stack.len());
                            continue 'outer;
                        }
                    }

                    // No match found, mark as unmatched
                    match_.stack_height = None;
                }
            }
        }

        // Remaining items in stack must be unmatched
        for match_ in stack.iter_mut() {
            match_.stack_height = None;
        }
    }

    pub fn line_matches(&self, line_number: usize) -> Option<Vec<Match>> {
        self.matches_by_line.get(line_number).cloned()
    }

    pub fn iter_from(
        &self,
        line_number: usize,
        col: usize,
    ) -> impl Iterator<Item = MatchWithLine> + '_ {
        self.matches_by_line[line_number.max(0)..]
            .iter()
            .enumerate()
            .flat_map(move |(offset, matches)| {
                let current_line = line_number + offset;
                matches
                    .iter()
                    .filter(move |match_| current_line != line_number || match_.col >= col)
                    .map(move |match_| match_.with_line(current_line))
            })
    }

    pub fn iter_to(
        &self,
        line_number: usize,
        col: usize,
    ) -> impl Iterator<Item = MatchWithLine> + '_ {
        self.matches_by_line[0..=line_number.min(self.matches_by_line.len() - 1)]
            .iter()
            .enumerate()
            .rev()
            .flat_map(move |(current_line, matches)| {
                matches
                    .iter()
                    .filter(move |match_| current_line != line_number || match_.col < col)
                    .map(move |match_| match_.with_line(current_line))
            })
    }

    pub fn span_at(&self, line_number: usize, col: usize) -> Option<String> {
        let line_matches = self.matches_by_line.get(line_number)?;
        let line_state = self.state_by_line.get(line_number)?;

        // Look for spans starting in the current line before the desired column

        let matching_span = line_matches
            .iter()
            .rev()
            // Get all opening matches before the cursor on the current line
            .filter(|match_| match_.kind == Kind::Opening && match_.col <= col)
            // Find closing match on the same line or no match (overflows to next line)
            .find_map(|opening| {
                match opening.token {
                    Token::InlineSpan(span, _, _) | Token::BlockSpan(span, _, _) => {
                        let closing = line_matches.iter().find(|closing| {
                            closing.kind == Kind::Closing
                                && closing.col > opening.col
                                && closing.token == opening.token
                                && closing.stack_height == opening.stack_height
                        });

                        match closing {
                            // Ends before desired column
                            Some(closing) if closing.col < col => None,
                            // Extends to end of line or found closing after desired column
                            _ => Some(span),
                        }
                    }
                    _ => None,
                }
            });

        if let Some(span) = matching_span {
            return Some(span.to_string());
        }

        // Look for spans that started before the current line
        match line_state {
            // TODO: check that the span doesn't end before the cursor
            State::InInlineSpan(span) | State::InBlockSpan(span) => Some(span.to_string()),
            _ => None,
        }
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

        // Ignore unmatched delimiter
        if matches!(match_at_pos.token, Token::Delimiter(_, _))
            && match_at_pos.stack_height.is_none()
        {
            return None;
        }

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
                            (line_number != matches_line_number || match_.col < col)
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

    pub fn stack_height_at(&self, line_number: usize, col: usize) -> usize {
        // Forward pass
        self.iter_from(line_number, col)
            .find_map(|match_| {
                match_.stack_height.map(|stack_height| {
                    stack_height + (if match_.kind == Kind::Closing { 1 } else { 0 })
                })
            })
            // Backward pass, if needed
            .or_else(|| {
                self.iter_to(line_number, col).find_map(|match_| {
                    match_.stack_height.map(|stack_height| {
                        stack_height + (if match_.kind == Kind::Opening { 1 } else { 0 })
                    })
                })
            })
            .unwrap_or(0)
    }

    pub fn unmatched_opening_before(
        &self,
        opening: &str,
        closing: &str,
        line_number: usize,
        col: usize,
    ) -> Option<MatchWithLine> {
        let cursor_stack_height = self.stack_height_at(line_number, col);
        let mut lowest_stack_height = cursor_stack_height;
        let mut current_stack_height = cursor_stack_height;

        for match_ in self
            .iter_to(line_number, col)
            .filter(|match_| matches!(match_.token, Token::Delimiter(_, _)))
        {
            if let Some(stack_height) = match_.stack_height {
                // Stack height higher than cursor
                if stack_height < lowest_stack_height {
                    // For example: ( [] ( | )
                    // Stack:            ^   ^
                    // Cursor stack height: 1
                    // We can close the outer pair by adding a closing pair at the cursor
                    if match_.kind == Kind::Opening
                        && match_.token.closing() == Some(closing)
                        && match_.token.opening() == opening
                    {
                        lowest_stack_height = stack_height;
                    }
                    // In this example: ( [ ( | ) ] )
                    // Stack:             ^ ^   ^ ^
                    // Cursor stack height: 2
                    // Inserting a closing pair would not close the outer pair, so we exit
                    else {
                        return None;
                    }
                }

                current_stack_height =
                    stack_height + if match_.kind == Kind::Closing { 1 } else { 0 };
            }

            // Unmatched opening with the same stack height
            if match_.kind == Kind::Opening
                && match_.token.opening() == opening
                && match_.token.closing() == Some(closing)
                && match_.stack_height == None
                && current_stack_height == lowest_stack_height
            {
                return Some(match_);
            }
        }

        None
    }

    pub fn unmatched_closing_after(
        &self,
        opening: &str,
        closing: &str,
        line_number: usize,
        col: usize,
    ) -> Option<MatchWithLine> {
        let cursor_stack_height = self.stack_height_at(line_number, col);
        let mut lowest_stack_height = cursor_stack_height;
        let mut current_stack_height = cursor_stack_height;

        for match_ in self
            .iter_from(line_number, col)
            .filter(|match_| matches!(match_.token, Token::Delimiter(_, _)))
        {
            if let Some(stack_height) = match_.stack_height {
                // Stack height higher than cursor
                if stack_height < lowest_stack_height {
                    // For example: ( | ) )
                    // Stack:       ^   ^
                    // Cursor stack height: 1
                    // We can close the outer pair by adding a closing pair at the cursor
                    if match_.kind == Kind::Closing
                        && match_.token.closing() == Some(closing)
                        && match_.token.opening() == opening
                    {
                        lowest_stack_height = stack_height;
                    }
                    // In this example: [ ( | ) ] )
                    // Stack:           ^ ^   ^ ^
                    // Cursor stack height: 2
                    // Inserting a closing pair would not close the outer pair, so we exit
                    else {
                        return None;
                    }
                }

                current_stack_height =
                    stack_height + if match_.kind == Kind::Opening { 1 } else { 0 };
            }

            // Unmatched closing with the same stack height
            if match_.kind == Kind::Closing
                && match_.token.opening() == opening
                && match_.token.closing() == Some(closing)
                && match_.stack_height == None
                && current_stack_height == lowest_stack_height
            {
                return Some(match_);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unmatched_opening_before() {
        let mut buffer = ParsedBuffer::parse("rust", &["("]).unwrap();
        assert_eq!(buffer.unmatched_opening_before("(", ")", 0, 0), None);
        assert_eq!(
            buffer.unmatched_opening_before("(", ")", 0, 1),
            Some(Match::delimiter('(', 0, None).with_line(0))
        );

        let mut buffer = ParsedBuffer::parse("rust", &["( ( )"]).unwrap();
        assert_eq!(
            buffer.unmatched_opening_before("(", ")", 0, 4),
            Some(Match::delimiter('(', 0, None).with_line(0))
        );
    }

    #[test]
    fn test_get_unmatched_closing_at() {
        let mut buffer = ParsedBuffer::parse("rust", &[")"]).unwrap();
        assert_eq!(
            buffer.unmatched_closing_after("(", ")", 0, 0),
            Some(Match::delimiter(')', 0, None).with_line(0))
        );
        assert_eq!(buffer.unmatched_closing_after("(", ")", 0, 1), None);
        assert_eq!(buffer.unmatched_closing_after("(", ")", 1, 1), None);

        let mut buffer = ParsedBuffer::parse("rust", &[" )"]).unwrap();
        assert_eq!(
            buffer.unmatched_closing_after("(", ")", 0, 0),
            Some(Match::delimiter(')', 1, None).with_line(0))
        );
        assert_eq!(
            buffer.unmatched_closing_after("(", ")", 0, 1),
            Some(Match::delimiter(')', 1, None).with_line(0))
        );
        assert_eq!(buffer.unmatched_closing_after("(", ")", 0, 2), None);
        assert_eq!(buffer.unmatched_closing_after("(", ")", 1, 0), None);

        let mut buffer = ParsedBuffer::parse("rust", &["( ] )"]).unwrap();
        assert_eq!(buffer.unmatched_closing_after("[", "]", 0, 0), None);
        assert_eq!(
            buffer.unmatched_closing_after("[", "]", 0, 1),
            Some(Match::delimiter(']', 2, None).with_line(0))
        );
    }
}
