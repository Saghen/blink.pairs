use crate::parser::{parse_filetype, Match, ParseState};

pub struct ParsedBuffer {
    matches_by_line: Vec<Vec<Match>>,
    state_by_line: Vec<ParseState>,
}

impl ParsedBuffer {
    pub fn parse(filetype: &str, text: &str) -> Option<Self> {
        let (matches_by_line, state_by_line) = parse_filetype(filetype, text, ParseState::Normal)?;

        Some(Self {
            matches_by_line,
            state_by_line,
        })
    }

    pub fn reparse_range(
        &mut self,
        filetype: &str,
        text: &str,
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

        if let Some((matches_by_line, state_by_line)) =
            parse_filetype(filetype, text, initial_state)
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

    fn recalculate_stack_heights(&mut self) {
        let mut stack = vec![];

        for matches in self.matches_by_line.iter_mut() {
            for match_ in matches {
                match &match_.closing {
                    // Opening delimiter
                    Some(closing) => {
                        match_.stack_height = stack.len();
                        stack.push(closing);
                    }
                    // Closing delimiter
                    None => {
                        if let Some(closing) = stack.last() {
                            if *closing == &match_.text {
                                stack.pop();
                            }
                        }
                        match_.stack_height = stack.len();
                    }
                }
            }
        }
    }
}
