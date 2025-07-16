use crate::parser::{parse_filetype, Kind, Match, MatchWithLine, State, Token};

pub struct ParsedBuffer {
    pub matches_by_line: Vec<Vec<Match>>,
    pub state_by_line: Vec<State>,
    pub indent_levels: Vec<u8>,
}

impl ParsedBuffer {
    pub fn parse(filetype: &str, tab_width: u8, lines: &[&str]) -> Option<Self> {
        let mut parsed = parse_filetype(filetype, tab_width, lines, State::Normal)?;
        parsed.calculate_stack_heights(tab_width);
        Some(parsed)
    }

    pub fn reparse_range(
        &mut self,
        filetype: &str,
        tab_width: u8,
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

        if let Some(new) = parse_filetype(filetype, tab_width, lines, initial_state) {
            let new_end_line = new_end_line.unwrap_or(start_line + new.matches_by_line.len());
            let length = new_end_line - start_line;

            self.matches_by_line.splice(
                start_line..old_end_line,
                new.matches_by_line[0..length].to_vec(),
            );
            self.state_by_line.splice(
                start_line..old_end_line,
                new.state_by_line[0..length].to_vec(),
            );
            self.indent_levels.splice(
                start_line..old_end_line.min(self.indent_levels.len()),
                new.indent_levels[0..length].to_vec(),
            );

            self.calculate_stack_heights(tab_width);

            true
        } else {
            false
        }
    }

    // TODO: return iterator
    fn recursive_find(
        &self,
        token: Token,
        line: usize,
        col: usize,
        stack_height: usize,
    ) -> Vec<MatchWithLine> {
        let matches: Vec<MatchWithLine> = self
            .iter_from(line, col + 1)
            .take_while(|match_| {
                match_
                    .stack_height
                    .map(|sh| sh >= stack_height)
                    .unwrap_or(true)
            })
            .filter(|match_| match_.stack_height == Some(stack_height) && match_.token == token)
            .collect();

        let mut all_matches = Vec::new();

        for matched_opening in matches {
            // Add the current match
            all_matches.push(matched_opening.clone());

            // Recursively find deeper matches
            let deeper_matches = self.recursive_find(
                token.clone(),
                matched_opening.line,
                matched_opening.col,
                stack_height + 1,
            );

            // Add all deeper matches
            all_matches.extend(deeper_matches);
        }

        all_matches
    }

    fn calculate_stack_heights(&mut self, tab_width: u8) {
        let mut unmatched_openings: Vec<(usize, usize)> = vec![];
        let mut stack = vec![];

        // Get stack heights for all openings using a traditional stack
        // This results in matching on the closest pairs when there are mismatched
        // openings/closings
        // [ ( ( [] (  ) ]
        // 0     11 1  1 0
        for (line, matches) in self.matches_by_line.iter_mut().enumerate() {
            'outer: for match_ in matches.iter_mut() {
                // Opening delimiter
                if match_.kind == Kind::Opening {
                    stack.push((line, match_));
                }
                // Closing delimiter
                else {
                    for (i, (_, opening)) in stack.iter().enumerate().rev() {
                        if opening.token == match_.token {
                            // Mark all skipped matches as unmatched
                            for (unmatched_line, unmatched_opening) in
                                stack.splice((i + 1).., vec![])
                            {
                                unmatched_openings.push((unmatched_line, unmatched_opening.col));
                            }

                            // Update stack height
                            let (_, opening) = stack.pop().unwrap();
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
        for (line, match_) in stack.into_iter() {
            unmatched_openings.push((line, match_.col));
        }
        unmatched_openings.sort();

        // Remove stack heights for unmatched openings
        for (line, col) in unmatched_openings.iter() {
            let match_ = self.match_at_mut(*line, *col).unwrap();
            match_.stack_height = None;
        }

        // Prefer matching on the furthest pair for mismatched openings
        // As is, we have matched like so:
        // [ ( ( [] (  ) ]
        // 0     11 1  1 0
        // but we want to match like:
        // [ ( ( [] (  ) ]
        // 0 1   22    1 0
        for (line, col) in unmatched_openings.into_iter().rev() {
            self.rematch_by_indent_recursive(line, col, tab_width);
        }
    }

    /// Gets the indent level of the line, rounded down to the nearest tab width
    pub fn nearest_indent_level(&self, line: usize, tab_width: u8) -> u8 {
        self.indent_levels[line].div_floor(tab_width) * tab_width
    }

    /// Given an unmatched opening's position, attempts to find a matching opening/closing pair with an
    /// where the closing ident level matches the unmatched opening.
    /// Performed recursively until the match cannot be moved further down the stack.
    ///
    /// ```rust
    /// if some_example {
    ///     //          ^ unmatched
    ///     if no_closing_on_this {
    ///         //       matched  ^
    /// }
    /// ```
    /// becomes
    /// ```rust
    /// if some_example {
    ///     //  matched ^
    ///     if no_closing_on_this {
    ///         //      unmatched ^
    ///     }
    /// }
    /// ```
    pub fn rematch_by_indent_recursive(&mut self, line: usize, col: usize, tab_width: u8) {
        let indent_level = self.nearest_indent_level(line, tab_width);
        let token = self.match_at(line, col).unwrap().token;
        let stack_height = self.stack_height_at(line, col);

        // Find the first matched opening that has the same stack height and token
        let mut matched_openings = self
            .recursive_find(token.clone(), line, col, stack_height)
            .iter()
            .flat_map(|match_| self.match_pair(match_.line, match_.col))
            .filter(|(open, close)| {
                self.nearest_indent_level(close.line, tab_width) == indent_level
                    && self.nearest_indent_level(close.line, tab_width)
                        != self.nearest_indent_level(open.line, tab_width)
            })
            .collect::<Vec<_>>();
        matched_openings.sort_by_key(|(_, close)| close.line);

        if let Some((matched_opening_with_line, matched_closing_with_line)) =
            matched_openings.first()
        {
            // Mark matched opening as unmatched
            let matched_opening = self
                .match_at_mut(
                    matched_opening_with_line.line,
                    matched_opening_with_line.col,
                )
                .unwrap();
            matched_opening.stack_height = None;

            // Mark unmatched opening as matched, using the stack height - 1 as unmatched
            // openings lead to incorrect stack heights for the matches after them
            // For example:
            // [ ( ( ) ]
            // 0   2 2 0
            // When it should be:
            // [ ( ( ) ]
            // 0   1 1 0
            // But since we're now matching on the unmatched opening, we end up with:
            // [ ( ( ) ]
            // 0 1   1 0
            let unmatched_opening = self.match_at_mut(line, col).unwrap();
            unmatched_opening.stack_height = Some(stack_height.saturating_sub(1));

            let matched_closing = self
                .match_at_mut(
                    matched_closing_with_line.line,
                    matched_closing_with_line.col,
                )
                .unwrap();
            matched_closing.stack_height = Some(stack_height.saturating_sub(1));

            // All matches after the closing match are now 1 stack height shallower,
            // For example, starting with:
            // [ ( ( ) { } ]
            // 0   2 2 2 2 0
            // After the previous step, we have:
            // [ ( ( ) { } ]
            // 0 1   1 2 2 0
            // So we update the "{ }" stack height by 1
            // [ ( ( ) { } ]
            // 0 1   1 1 1 0
            for match_ in self.matches_by_line[matched_closing_with_line.line..]
                .iter_mut()
                .enumerate()
                .flat_map(|(line_idx, matches)| {
                    matches.iter_mut().filter(move |match_| {
                        line_idx != 0 || match_.col > matched_closing_with_line.col
                    })
                })
            {
                if match_.stack_height == Some(stack_height.saturating_sub(1))
                    && match_.kind == Kind::Closing
                {
                    break;
                }
                match_.stack_height = match_
                    .stack_height
                    .map(|stack_height| stack_height.saturating_sub(1));
            }

            self.rematch_by_indent_recursive(
                matched_opening_with_line.line,
                matched_opening_with_line.col,
                tab_width,
            );
        }
    }

    pub fn line_matches(&self, line_number: usize) -> Option<Vec<Match>> {
        self.matches_by_line.get(line_number).cloned()
    }

    pub fn get_indent_levels(&self, start_line: usize, end_line: usize) -> Vec<u8> {
        let start_idx = start_line.min(self.indent_levels.len());
        let end_idx = end_line.min(self.indent_levels.len());

        if start_idx >= end_idx {
            return vec![];
        }

        self.indent_levels[start_idx..end_idx].to_vec()
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

    pub fn match_at_mut(&mut self, line_number: usize, col: usize) -> Option<&mut Match> {
        self.matches_by_line
            .get_mut(line_number)?
            .iter_mut()
            .find(|match_| (match_.col..(match_.col + match_.len())).contains(&col))
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

    pub fn stack_height_at_forward(&self, line_number: usize, col: usize) -> Option<usize> {
        self.iter_from(line_number, col).find_map(|match_| {
            match_.stack_height.map(|stack_height| {
                stack_height + (if match_.kind == Kind::Closing { 1 } else { 0 })
            })
        })
    }

    pub fn stack_height_at_backward(&self, line_number: usize, col: usize) -> Option<usize> {
        self.iter_to(line_number, col).find_map(|match_| {
            match_.stack_height.map(|stack_height| {
                stack_height + (if match_.kind == Kind::Opening { 1 } else { 0 })
            })
        })
    }

    pub fn stack_height_at(&self, line_number: usize, col: usize) -> usize {
        self.stack_height_at_forward(line_number, col)
            .or_else(|| self.stack_height_at_backward(line_number, col))
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

    fn parse(filetype: &str, lines: &[&str]) -> ParsedBuffer {
        ParsedBuffer::parse(filetype, 4, lines).unwrap()
    }

    #[test]
    fn test_unmatched_opening_before() {
        let buffer = parse("rust", &["("]);
        assert_eq!(buffer.unmatched_opening_before("(", ")", 0, 0), None);
        assert_eq!(
            buffer.unmatched_opening_before("(", ")", 0, 1),
            Some(Match::delimiter('(', 0, None).with_line(0))
        );

        let buffer = parse("rust", &["( ( )"]);
        assert_eq!(
            buffer.unmatched_opening_before("(", ")", 0, 4),
            Some(Match::delimiter('(', 0, None).with_line(0))
        );
    }

    #[test]
    fn test_get_unmatched_closing_at() {
        let buffer = parse("rust", &[")"]);
        assert_eq!(
            buffer.unmatched_closing_after("(", ")", 0, 0),
            Some(Match::delimiter(')', 0, None).with_line(0))
        );
        assert_eq!(buffer.unmatched_closing_after("(", ")", 0, 1), None);
        assert_eq!(buffer.unmatched_closing_after("(", ")", 1, 1), None);

        let buffer = parse("rust", &[" )"]);
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

        let buffer = parse("rust", &["( ] )"]);
        assert_eq!(buffer.unmatched_closing_after("[", "]", 0, 0), None);
        assert_eq!(
            buffer.unmatched_closing_after("[", "]", 0, 1),
            Some(Match::delimiter(']', 2, None).with_line(0))
        );
    }
}
