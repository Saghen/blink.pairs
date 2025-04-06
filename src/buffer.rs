use crate::languages::TokenType;
use crate::parser::{parse_filetype, Match, MatchWithLine, ParseState, Span};
use std::collections::HashMap;

pub struct ParsedBuffer {
    matches_by_line: Vec<Vec<Match>>,
    state_by_line: Vec<ParseState>,
    spans: Vec<Span>,
    span_by_id: HashMap<usize, usize>,
}

impl ParsedBuffer {
    pub fn parse(filetype: &str, lines: &[&str]) -> Option<Self> {
        let (matches_by_line, state_by_line, spans) =
            parse_filetype(filetype, lines, ParseState::Normal)?;

        let span_by_id = Self::build_span_lookup(&spans);

        Some(Self {
            matches_by_line,
            state_by_line,
            spans,
            span_by_id,
        })
    }

    #[inline]
    fn build_span_lookup(spans: &[Span]) -> HashMap<usize, usize> {
        let mut span_by_id = HashMap::with_capacity(spans.len());
        for (i, span) in spans.iter().enumerate() {
            span_by_id.insert(span.id, i);
        }
        span_by_id
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
                .unwrap_or(ParseState::Normal)
        } else {
            ParseState::Normal
        };

        if let Some((matches_by_line, state_by_line, spans)) =
            parse_filetype(filetype, lines, initial_state)
        {
            let new_end_line = new_end_line.unwrap_or(start_line + matches_by_line.len());
            let length = new_end_line - start_line;

            if old_end_line != start_line || length > 0 {
                self.matches_by_line.splice(
                    start_line..old_end_line,
                    matches_by_line.into_iter().take(length),
                );
                self.state_by_line.splice(
                    start_line..old_end_line,
                    state_by_line.into_iter().take(length),
                );
            }

            self.spans = spans;
            self.span_by_id = Self::build_span_lookup(&self.spans);
            self.recalculate_stack_heights();

            true
        } else {
            false
        }
    }

    pub fn line_matches(&self, line_number: usize) -> Option<&[Match]> {
        self.matches_by_line.get(line_number).map(|v| v.as_slice())
    }

    pub fn match_at(&self, line_number: usize, col: usize) -> Option<Match> {
        if line_number >= self.matches_by_line.len() {
            return None;
        }

        let line_matches = &self.matches_by_line[line_number];

        for match_ in line_matches {
            if match_.type_ == TokenType::Delimiter && match_.col == col {
                return Some(match_.clone());
            }
        }

        for match_ in line_matches {
            let start_col = match_.col;
            let end_col = start_col + match_.text.len();

            if col >= start_col && col < end_col {
                return Some(match_.clone());
            }
        }

        for match_ in line_matches {
            if match_.type_ == TokenType::Delimiter
                && match_.text.len() == 1
                && match_.col == col.saturating_sub(1)
            {
                return Some(match_.clone());
            }
        }

        for match_ in line_matches {
            if match_.col == col && match_.text.is_empty() && match_.span_name.is_some() {
                return Some(match_.clone());
            }
        }

        for match_ in line_matches {
            if match_.col == 0
                && match_.text.is_empty()
                && match_.span_name.is_some()
                && match_.type_ == TokenType::BlockSpan
            {
                let mut marker = match_.clone();
                marker.col = col;
                return Some(marker);
            }
        }

        if let Some(ParseState::InBlockSpan { name, span_id, .. }) =
            &self.state_by_line.get(line_number)
        {
            return Some(Match {
                type_: TokenType::BlockSpan,
                text: "",
                col,
                closing: None,
                stack_height: None,
                span_name: Some(*name),
                span_id: Some(*span_id),
            });
        }

        self.find_span_at_position(line_number, col)
    }

    #[inline]
    fn find_span_at_position(&self, line_number: usize, col: usize) -> Option<Match> {
        for span in &self.spans {
            if let Some(closing) = &span.closing {
                let start_line = span.opening.line;
                let start_col = span.opening.col + span.opening.text.len();
                let end_line = closing.line;
                let end_col = closing.col;

                if line_number < start_line || line_number > end_line {
                    continue;
                }

                if start_line == end_line {
                    if line_number == start_line && col >= start_col && col < end_col {
                        return Some(Match {
                            type_: span.type_,
                            text: "",
                            col,
                            closing: None,
                            stack_height: None,
                            span_name: Some(span.name),
                            span_id: Some(span.id),
                        });
                    }
                } else if (line_number == start_line && col >= start_col)
                    || (line_number == end_line && col < end_col)
                    || (line_number > start_line && line_number < end_line)
                {
                    return Some(Match {
                        type_: span.type_,
                        text: "",
                        col,
                        closing: None,
                        stack_height: None,
                        span_name: Some(span.name),
                        span_id: Some(span.id),
                    });
                }
            }
        }
        None
    }

    pub fn match_pair(
        &self,
        line_number: usize,
        col: usize,
    ) -> Option<(MatchWithLine, MatchWithLine)> {
        let match_at_pos = self.match_at(line_number, col)?.with_line(line_number);

        if let Some(span_id) = match_at_pos.span_id {
            if let Some(&span_idx) = self.span_by_id.get(&span_id) {
                let span = &self.spans[span_idx];

                // Check if we're on opening token
                if match_at_pos.col == span.opening.col && match_at_pos.line == span.opening.line {
                    // Return pair if span has closing token
                    if let Some(closing) = &span.closing {
                        let closing_match = Match {
                            type_: match_at_pos.type_,
                            text: closing.text,
                            col: closing.col,
                            closing: None,
                            stack_height: match_at_pos.stack_height,
                            span_name: match_at_pos.span_name,
                            span_id: match_at_pos.span_id,
                        }
                        .with_line(closing.line);

                        return Some((match_at_pos, closing_match));
                    }
                }
                // Check if we're on closing token
                else if let Some(closing) = &span.closing {
                    if match_at_pos.col == closing.col && match_at_pos.line == closing.line {
                        let opening_match = Match {
                            type_: match_at_pos.type_,
                            text: span.opening.text,
                            col: span.opening.col,
                            closing: Some(closing.text),
                            stack_height: match_at_pos.stack_height,
                            span_name: match_at_pos.span_name,
                            span_id: match_at_pos.span_id,
                        }
                        .with_line(span.opening.line);

                        return Some((opening_match, match_at_pos));
                    }
                }
                // Inside span text (not on a token)
                else {
                    let opening = &span.opening;
                    let closing_text = span.closing.as_ref().map(|c| c.text);

                    let opening_match = Match {
                        type_: match_at_pos.type_,
                        text: opening.text,
                        col: opening.col,
                        closing: closing_text,
                        stack_height: None,
                        span_name: match_at_pos.span_name,
                        span_id: match_at_pos.span_id,
                    }
                    .with_line(opening.line);

                    if let Some(closing) = &span.closing {
                        let closing_match = Match {
                            type_: match_at_pos.type_,
                            text: closing.text,
                            col: closing.col,
                            closing: None,
                            stack_height: None,
                            span_name: match_at_pos.span_name,
                            span_id: match_at_pos.span_id,
                        }
                        .with_line(closing.line);

                        return Some((opening_match, closing_match));
                    }
                }
            }
        }

        // For delimiters or when span lookup failed
        if match_at_pos.closing.is_some() {
            self.find_closing_match(match_at_pos)
        } else {
            self.find_opening_match(match_at_pos)
        }
    }

    fn find_closing_match(
        &self,
        match_at_pos: MatchWithLine,
    ) -> Option<(MatchWithLine, MatchWithLine)> {
        let line = match_at_pos.line;
        let col = match_at_pos.col;
        let expected_closing = match_at_pos.closing?;
        let stack_height = match_at_pos.stack_height;

        if let Some(matches) = self.matches_by_line.get(line) {
            for match_ in matches {
                if match_.col > col
                    && match_.type_ == match_at_pos.type_
                    && match_.text == expected_closing
                    && match_.stack_height == stack_height
                {
                    return Some((match_at_pos, match_.with_line(line)));
                }
            }
        }

        for curr_line in (line + 1)..self.matches_by_line.len() {
            for match_ in &self.matches_by_line[curr_line] {
                if match_.type_ == match_at_pos.type_
                    && match_.text == expected_closing
                    && match_.stack_height == stack_height
                {
                    return Some((match_at_pos, match_.with_line(curr_line)));
                }
            }
        }

        None
    }

    fn find_opening_match(
        &self,
        match_at_pos: MatchWithLine,
    ) -> Option<(MatchWithLine, MatchWithLine)> {
        let line = match_at_pos.line;
        let col = match_at_pos.col;
        let stack_height = match_at_pos.stack_height;

        if let Some(matches) = self.matches_by_line.get(line) {
            for match_ in matches {
                if match_.col < col
                    && match_.type_ == match_at_pos.type_
                    && match_.closing.as_ref() == Some(&match_at_pos.text)
                    && match_.stack_height == stack_height
                {
                    return Some((match_.with_line(line), match_at_pos.clone()));
                }
            }
        }

        for curr_line in (0..line).rev() {
            for match_ in &self.matches_by_line[curr_line] {
                if match_.type_ == match_at_pos.type_
                    && match_.closing.as_ref() == Some(&match_at_pos.text)
                    && match_.stack_height == stack_height
                {
                    return Some((match_.with_line(curr_line), match_at_pos.clone()));
                }
            }
        }

        None
    }

    fn recalculate_stack_heights(&mut self) {
        let mut stack = Vec::with_capacity(32);

        for matches in &mut self.matches_by_line {
            for match_ in matches
                .iter_mut()
                .filter(|m| m.type_ == TokenType::Delimiter)
            {
                match &match_.closing {
                    Some(closing) => {
                        match_.stack_height = Some(stack.len());
                        stack.push(closing);
                    }
                    None => {
                        if let Some(closing) = stack.last() {
                            if *closing == &match_.text {
                                stack.pop();
                            }
                        }
                        match_.stack_height = Some(stack.len());
                    }
                }
            }
        }

        let mut span_depths: HashMap<usize, usize> = HashMap::with_capacity(self.spans.len());

        for span in &self.spans {
            if span.parent_id.is_none() {
                span_depths.insert(span.id, 0);
                self.calculate_span_depths(span.id, 0, &mut span_depths);
            }
        }

        for matches in &mut self.matches_by_line {
            for match_ in matches
                .iter_mut()
                .filter(|m| m.type_ == TokenType::InlineSpan || m.type_ == TokenType::BlockSpan)
            {
                if let Some(span_id) = match_.span_id {
                    if let Some(&depth) = span_depths.get(&span_id) {
                        match_.stack_height = Some(depth);
                    }
                }
            }
        }
    }

    #[inline]
    fn calculate_span_depths(
        &self,
        span_id: usize,
        depth: usize,
        depths: &mut HashMap<usize, usize>,
    ) {
        if let Some(&span_idx) = self.span_by_id.get(&span_id) {
            let span = &self.spans[span_idx];
            let child_depth = depth + 1;

            for &child_id in &span.children {
                depths.insert(child_id, child_depth);
                self.calculate_span_depths(child_id, child_depth, depths);
            }
        }
    }

    pub fn get_state_at_line(&self, line_number: usize) -> Option<&ParseState> {
        self.state_by_line.get(line_number)
    }

    pub fn get_spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn get_span_by_id(&self, span_id: usize) -> Option<&Span> {
        self.span_by_id.get(&span_id).map(|&idx| &self.spans[idx])
    }
}
