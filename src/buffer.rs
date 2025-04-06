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

        // Build span_by_id lookup
        let mut span_by_id = HashMap::new();
        for (i, span) in spans.iter().enumerate() {
            span_by_id.insert(span.id, i);
        }

        Some(Self {
            matches_by_line,
            state_by_line,
            spans,
            span_by_id,
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
                .unwrap_or(ParseState::Normal)
        } else {
            ParseState::Normal
        };

        if let Some((matches_by_line, state_by_line, spans)) =
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

            // Update spans - we'll replace all spans completely for simplicity
            self.spans = spans;

            // Rebuild span_by_id lookup
            self.span_by_id.clear();
            for (i, span) in self.spans.iter().enumerate() {
                self.span_by_id.insert(span.id, i);
            }

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
        let line_matches = self.matches_by_line.get(line_number)?;

        let exact_match = line_matches
            .iter()
            .find(|match_| (match_.col..(match_.col + match_.text.len())).contains(&col))
            .cloned();

        if exact_match.is_some() {
            return exact_match;
        }

        let virtual_match = line_matches
            .iter()
            .find(|match_| {
                match_.col == col && match_.text.is_empty() && match_.span_name.is_some()
            })
            .cloned();

        if virtual_match.is_some() {
            return virtual_match;
        }

        let block_span_marker = line_matches
            .iter()
            .find(|match_| {
                match_.col == 0
                    && match_.text.is_empty()
                    && match_.span_name.is_some()
                    && match_.type_ == TokenType::BlockSpan
            })
            .cloned();

        if let Some(mut marker) = block_span_marker {
            marker.col = col;
            return Some(marker);
        }

        if let Some(state) = self.state_by_line.get(line_number) {
            if let ParseState::InBlockSpan { name, span_id, .. } = state {
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
        }

        for span in &self.spans {
            if let Some(closing) = &span.closing {
                let start_line = span.opening.line;
                let end_line = closing.line;

                if line_number < start_line || line_number > end_line {
                    continue;
                }

                if line_number == start_line && col < span.opening.col + span.opening.text.len() {
                    continue;
                }

                if line_number == end_line && col >= closing.col {
                    continue;
                }

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

                if match_at_pos.closing.is_some() {
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
                } else if match_at_pos.closing.is_none() {
                    let opening = &span.opening;
                    let opening_match = Match {
                        type_: match_at_pos.type_,
                        text: opening.text,
                        col: opening.col,
                        closing: if let Some(closing) = &span.closing {
                            Some(closing.text)
                        } else {
                            None
                        },
                        stack_height: match_at_pos.stack_height,
                        span_name: match_at_pos.span_name,
                        span_id: match_at_pos.span_id,
                    }
                    .with_line(opening.line);

                    return Some((opening_match, match_at_pos));
                }
            }
        }

        if match_at_pos.type_ == TokenType::Delimiter {
            if match_at_pos.closing.is_some() {
                let closing_match = self.matches_by_line[line_number..]
                    .iter()
                    .enumerate()
                    .map(|(matches_line_number, matches)| {
                        (matches_line_number + line_number, matches)
                    })
                    .find_map(|(matches_line_number, matches)| {
                        matches
                            .iter()
                            .find(|match_| {
                                (line_number != matches_line_number || match_.col > col)
                                    && match_at_pos.type_ == match_.type_
                                    && match_at_pos.closing == Some(match_.text)
                                    && match_at_pos.stack_height == match_.stack_height
                            })
                            .map(|match_| match_.with_line(matches_line_number))
                    })?;

                return Some((match_at_pos, closing_match));
            } else {
                let opening_match = self.matches_by_line[0..(line_number + 1)]
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(matches_line_number, matches)| {
                        matches
                            .iter()
                            .rev()
                            .find(|match_| {
                                (line_number != matches_line_number || match_.col < col)
                                    && match_at_pos.type_ == match_.type_
                                    && Some(match_at_pos.text) == match_.closing
                                    && match_at_pos.stack_height == match_.stack_height
                            })
                            .map(|match_| match_.with_line(matches_line_number))
                    })?;

                return Some((opening_match, match_at_pos));
            }
        }

        None
    }

    fn recalculate_stack_heights(&mut self) {
        let mut stack = vec![];

        for matches in self.matches_by_line.iter_mut() {
            for match_ in matches {
                if match_.type_ != TokenType::Delimiter {
                    continue;
                }

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
