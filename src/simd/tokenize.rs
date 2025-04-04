use std::simd::cmp::SimdPartialEq;

use super::SimdVec;

#[derive(Debug, Clone, PartialEq)]
pub struct SimdToken {
    pub token: SimdTokenType,
    pub col: usize,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimdTokenType {
    None = 0,
    NewLine = 1,
    Escape = 2,

    CurlyBraceOpen = 3,
    CurlyBraceClose = 4,
    SquareBracketOpen = 5,
    SquareBracketClose = 6,
    ParenthesisOpen = 7,
    ParenthesisClose = 8,

    SingleQuote = 9,
    DoubleQuote = 10,
}

impl From<SimdTokenType> for u8 {
    fn from(val: SimdTokenType) -> Self {
        val as u8
    }
}

impl From<u8> for SimdTokenType {
    fn from(val: u8) -> Self {
        match val {
            0 => SimdTokenType::None,
            1 => SimdTokenType::NewLine,
            2 => SimdTokenType::Escape,
            3 => SimdTokenType::CurlyBraceOpen,
            4 => SimdTokenType::CurlyBraceClose,
            5 => SimdTokenType::SquareBracketOpen,
            6 => SimdTokenType::SquareBracketClose,
            7 => SimdTokenType::ParenthesisOpen,
            8 => SimdTokenType::ParenthesisClose,
            9 => SimdTokenType::SingleQuote,
            10 => SimdTokenType::DoubleQuote,
            _ => panic!("Invalid token value: {}", val),
        }
    }
}

pub fn tokenize(text: &str) -> impl Iterator<Item = SimdToken> + '_ {
    // Tokens
    let none_token = SimdVec::splat(SimdTokenType::None.into());

    let new_line_token = SimdVec::splat(SimdTokenType::NewLine.into());
    let new_line = SimdVec::splat(b'\n');

    let escape_token = SimdVec::splat(SimdTokenType::Escape.into());
    let escape = SimdVec::splat(b'\\');

    let curly_brace_open_token = SimdVec::splat(SimdTokenType::CurlyBraceOpen.into());
    let curly_brace_open = SimdVec::splat(b'{');

    let curly_brace_close_token = SimdVec::splat(SimdTokenType::CurlyBraceClose.into());
    let curly_brace_close = SimdVec::splat(b'}');

    let square_bracket_open_token = SimdVec::splat(SimdTokenType::SquareBracketOpen.into());
    let square_bracket_open = SimdVec::splat(b'[');

    let square_bracket_close_token = SimdVec::splat(SimdTokenType::SquareBracketClose.into());
    let square_bracket_close = SimdVec::splat(b']');

    let round_bracket_open_token = SimdVec::splat(SimdTokenType::ParenthesisOpen.into());
    let round_bracket_open = SimdVec::splat(b'(');

    let round_bracket_close_token = SimdVec::splat(SimdTokenType::ParenthesisClose.into());
    let round_bracket_close = SimdVec::splat(b')');

    let single_quote_token = SimdVec::splat(SimdTokenType::SingleQuote.into());
    let single_quote = SimdVec::splat(b'\'');

    let double_quote_token = SimdVec::splat(SimdTokenType::DoubleQuote.into());
    let double_quote = SimdVec::splat(b'"');

    //

    let mut col_offset = 0;
    text.as_bytes()
        .chunks(SimdVec::LEN)
        .map(SimdVec::load_or_default)
        .enumerate()
        .map(move |(idx, chunk)| {
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
            tokens
                .to_array()
                .into_iter()
                .enumerate()
                .map(move |(idx_in_chunk, token)| match token.into() {
                    SimdTokenType::None => None,
                    SimdTokenType::NewLine => {
                        col_offset += idx_in_chunk;

                        return Some(SimdToken {
                            token: token.into(),
                            col: chunk_col + idx_in_chunk + idx_in_chunk - col_offset,
                        });
                    }
                    token => Some(SimdToken {
                        token: token.into(),
                        col: chunk_col + idx_in_chunk - col_offset,
                    }),
                })
                .flatten()
        })
        .flatten()
}
