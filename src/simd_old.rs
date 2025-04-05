use std::simd::{cmp::SimdPartialEq, Simd};

type SimdVec = Simd<u8, 32>;


pub fn tokenize(lines: &[&str]) -> Option<Vec<Vec<SimdMatch>>> {
    // Tokens
    let none_token = SimdVec::splat(SimdToken::None.into());

    let new_line_token = SimdVec::splat(SimdToken::NewLine.into());
    let new_line = SimdVec::splat(b'\n');

    let escape_token = SimdVec::splat(SimdToken::Escape.into());
    let escape = SimdVec::splat(b'\\');

    let curly_brace_open_token = SimdVec::splat(SimdToken::CurlyBraceOpen.into());
    let curly_brace_open = SimdVec::splat(b'{');

    let curly_brace_close_token = SimdVec::splat(SimdToken::CurlyBraceClose.into());
    let curly_brace_close = SimdVec::splat(b'}');

    let square_bracket_open_token = SimdVec::splat(SimdToken::SquareBracketOpen.into());
    let square_bracket_open = SimdVec::splat(b'[');

    let square_bracket_close_token = SimdVec::splat(SimdToken::SquareBracketClose.into());
    let square_bracket_close = SimdVec::splat(b']');

    let round_bracket_open_token = SimdVec::splat(SimdToken::ParenthesisOpen.into());
    let round_bracket_open = SimdVec::splat(b'(');

    let round_bracket_close_token = SimdVec::splat(SimdToken::ParenthesisClose.into());
    let round_bracket_close = SimdVec::splat(b')');

    let single_quote_token = SimdVec::splat(SimdToken::SingleQuote.into());
    let single_quote = SimdVec::splat(b'\'');

    let double_quote_token = SimdVec::splat(SimdToken::DoubleQuote.into());
    let double_quote = SimdVec::splat(b'"');

    // State

    let mut matches_by_line = Vec::with_capacity(lines.len());
    matches_by_line.push(vec![]);

    let mut escaped = false;

    let mut state = SimdState::Normal;

    for (idx, chunk) in lines
        .join("\n")
        .as_bytes()
        .chunks(SimdVec::LEN)
        .map(SimdVec::load_or_default)
        .enumerate()
    {
        let mut tokens = none_token;

        tokens = tokens | new_line.simd_eq(chunk).select(new_line_token, tokens);
        tokens = tokens | escape.simd_eq(chunk).select(escape_token, tokens);

        tokens = tokens
            | curly_brace_open
                .simd_eq(chunk)
                .select(curly_brace_open_token, tokens);
        tokens = tokens
            | curly_brace_close
                .simd_eq(chunk)
                .select(curly_brace_close_token, tokens);

        tokens = tokens
            | square_bracket_open
                .simd_eq(chunk)
                .select(square_bracket_open_token, tokens);
        tokens = tokens
            | square_bracket_close
                .simd_eq(chunk)
                .select(square_bracket_close_token, tokens);

        tokens = tokens
            | round_bracket_open
                .simd_eq(chunk)
                .select(round_bracket_open_token, tokens);
        tokens = tokens
            | round_bracket_close
                .simd_eq(chunk)
                .select(round_bracket_close_token, tokens);

        tokens = tokens
            | single_quote
                .simd_eq(chunk)
                .select(single_quote_token, tokens);
        tokens = tokens
            | double_quote
                .simd_eq(chunk)
                .select(double_quote_token, tokens);

        // Apply parsed tokens
        let chunk_col = idx * SimdVec::LEN;
        for (idx_in_chunk, token) in tokens.to_array().into_iter().enumerate() {
            use SimdState::*;

            match (&state, token.into(), escaped) {
                (_, SimdToken::None, _) => escaped = false,

                // Newline
                (_, SimdToken::NewLine, _) => {
                    matches_by_line.push(vec![]);
                    escaped = false;
                    if matches!(state, InString(_)) {
                        state = Normal;
                    }
                }
                // Escaped
                (_, SimdToken::Escape, _) => escaped = !escaped,

                // Strings
                (Normal, SimdToken::SingleQuote, false) => state = InString(SimdToken::SingleQuote),
                (InString(SimdToken::SingleQuote), SimdToken::SingleQuote, false) => state = Normal,
                (Normal, SimdToken::DoubleQuote, false) => state = InString(SimdToken::DoubleQuote),
                (InString(SimdToken::DoubleQuote), SimdToken::DoubleQuote, false) => state = Normal,

                // Delimiter
                (Normal, token, false) => matches_by_line.last_mut().unwrap().push(SimdMatch {
                    token,
                    col: chunk_col + idx_in_chunk,
                }),

                // No match
                _ => escaped = false,
            };
        }
    }

    Some(matches_by_line)
}

pub fn parse(lines: &[&str]) -> Option<Vec<Vec<SimdMatch>>> {
    // First pass: Identify all potential token characters using SIMD
    let potential_tokens = tokenize(lines)?;

    // Second pass: Combine adjacent tokens into multi-character tokens
    let mut result = Vec::with_capacity(potential_tokens.len());

    for line_tokens in potential_tokens {
        let mut combined_tokens = Vec::new();
        let mut i = 0;

        while i < line_tokens.len() {
            if i + 1 < line_tokens.len() {
                let current = &line_tokens[i];
                let next = &line_tokens[i + 1];

                // Check if current and next form a multi-character token
                if next.col == current.col + 1 {
                    match (current.token, next.token) {
                        (SimdToken::Minus, SimdToken::GreaterThan) => {
                            combined_tokens.push(SimdMatch {
                                token: SimdToken::Arrow,
                                col: current.col,
                            });
                            i += 2;
                            continue;
                        }
                        // Add other multi-character token combinations
                        _ => {}
                    }
                }
            }

            // No multi-character token formed, keep the single token
            combined_tokens.push(line_tokens[i].clone());
            i += 1;
        }

        result.push(combined_tokens);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curly_braces() {
        let lines = vec!["{}"];
        let matches_by_line = simd_parse_c(&lines).unwrap();
        assert_eq!(matches_by_line.len(), 1);
        assert_eq!(matches_by_line[0].len(), 2);
        assert_eq!(
            matches_by_line[0][0],
            SimdMatch {
                token: SimdToken::CurlyBraceOpen.into(),
                col: 0
            }
        );
        assert_eq!(
            matches_by_line[0][1],
            SimdMatch {
                token: SimdToken::CurlyBraceClose.into(),
                col: 1
            }
        )
    }

    #[test]
    fn test_ignore_strings() {
        let lines = vec!["\"{\""];
        let matches_by_line = simd_parse_c(&lines).unwrap();
        assert_eq!(matches_by_line.len(), 1);
        assert_eq!(matches_by_line[0].len(), 0);
    }
}
