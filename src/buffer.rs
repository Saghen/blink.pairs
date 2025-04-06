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
        if line_number >= self.matches_by_line.len() {
            return None;
        }
        
        let line_matches = self.matches_by_line.get(line_number)?;

        let exact_delimiter = line_matches
            .iter()
            .find(|match_| 
                match_.type_ == TokenType::Delimiter && 
                match_.col == col
            )
            .cloned();
            
        if exact_delimiter.is_some() {
            return exact_delimiter;
        }
        
        for match_ in line_matches {
            let start_col = match_.col;
            let end_col = start_col + match_.text.len();
            
            if col >= start_col && col < end_col {
                return Some(match_.clone());
            }
        }
        
        let adjacent_delimiter = line_matches
            .iter()
            .find(|match_| 
                match_.type_ == TokenType::Delimiter && 
                match_.text.len() == 1 &&
                match_.col == col.saturating_sub(1)
            )
            .cloned();
            
        if adjacent_delimiter.is_some() {
            return adjacent_delimiter;
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
                }
                else {
                    if line_number == start_line && col >= start_col {
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
                    else if line_number == end_line && col < end_col {
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
                    else if line_number > start_line && line_number < end_line {
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
        }

        None
    }

    pub fn match_pair(
        &self,
        line_number: usize,
        col: usize,
    ) -> Option<(MatchWithLine, MatchWithLine)> {
        let match_at_pos = self.match_at(line_number, col)?.with_line(line_number);

        // For spans, use the span ID to find the matching token
        if let Some(span_id) = match_at_pos.span_id {
            if let Some(&span_idx) = self.span_by_id.get(&span_id) {
                let span = &self.spans[span_idx];
                
                // Opening token
                if match_at_pos.col == span.opening.col && 
                   match_at_pos.line == span.opening.line {
                    // Only return if span has a closing token
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
                // Closing token
                else if let Some(closing) = &span.closing {
                    if match_at_pos.col == closing.col && 
                       match_at_pos.line == closing.line {
                        let opening = &span.opening;
                        let opening_match = Match {
                            type_: match_at_pos.type_,
                            text: opening.text,
                            col: opening.col,
                            closing: Some(closing.text),
                            stack_height: match_at_pos.stack_height,
                            span_name: match_at_pos.span_name,
                            span_id: match_at_pos.span_id,
                        }
                        .with_line(opening.line);
                        
                        return Some((opening_match, match_at_pos));
                    }
                }
                // Inside span text (not on a token) - find both opening and closing
                else {
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

        // For delimiters or when span lookup failed, fall back to the search approach
        // Opening delimiter
        if match_at_pos.closing.is_some() {
            // Find the closing match by searching forward
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
        } 
        // Closing delimiter
        else {
            // Find the opening match by searching backward
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
        
        let mut span_depths: HashMap<usize, usize> = HashMap::new();
        
        for span in &self.spans {
            if span.parent_id.is_none() {
                span_depths.insert(span.id, 0);
                
                self.calculate_span_depths(span.id, 0, &mut span_depths);
            }
        }
        
        for matches in self.matches_by_line.iter_mut() {
            for match_ in matches {
                if match_.type_ == TokenType::InlineSpan || match_.type_ == TokenType::BlockSpan {
                    if let Some(span_id) = match_.span_id {
                        if let Some(&depth) = span_depths.get(&span_id) {
                            match_.stack_height = Some(depth);
                        }
                    }
                }
            }
        }
    }
    
    fn calculate_span_depths(&self, span_id: usize, depth: usize, depths: &mut HashMap<usize, usize>) {
        if let Some(&span_idx) = self.span_by_id.get(&span_id) {
            let span = &self.spans[span_idx];
            
            for &child_id in &span.children {
                depths.insert(child_id, depth + 1);
                self.calculate_span_depths(child_id, depth + 1, depths);
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
