use std::simd::cmp::SimdPartialEq;

use super::{parse::SimdMatch, SimdVec};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimdToken {
    pub token: SimdTokenType,
    pub col: usize,
}

impl SimdToken {
    pub fn into_match(self, stack_height: Option<usize>) -> SimdMatch {
        SimdMatch {
            token: self.token.into(),
            col: self.col,
            stack_height,
        }
    }
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
    BackTick = 11,

    ForwardSlash = 12,
    Star = 13,
    Dash = 14,
    Dollar = 15,
    At = 16,
    Percent = 17,
    Hash = 18,
    Semicolon = 19,
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
            11 => SimdTokenType::BackTick,
            12 => SimdTokenType::ForwardSlash,
            13 => SimdTokenType::Star,
            14 => SimdTokenType::Dash,
            15 => SimdTokenType::Dollar,
            16 => SimdTokenType::At,
            17 => SimdTokenType::Percent,
            18 => SimdTokenType::Hash,
            19 => SimdTokenType::Semicolon,
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

    let backtick_token = SimdVec::splat(SimdTokenType::BackTick.into());
    let backtick = SimdVec::splat(b'`');

    let forward_slash_token = SimdVec::splat(SimdTokenType::ForwardSlash.into());
    let forward_slash = SimdVec::splat(b'/');

    let star_token = SimdVec::splat(SimdTokenType::Star.into());
    let star = SimdVec::splat(b'*');

    let dash_token = SimdVec::splat(SimdTokenType::Dash.into());
    let dash = SimdVec::splat(b'-');

    let dollar_token = SimdVec::splat(SimdTokenType::Dollar.into());
    let dollar = SimdVec::splat(b'$');

    let at_token = SimdVec::splat(SimdTokenType::At.into());
    let at = SimdVec::splat(b'@');

    let percent_token = SimdVec::splat(SimdTokenType::Percent.into());
    let percent = SimdVec::splat(b'%');

    let hash_token = SimdVec::splat(SimdTokenType::Hash.into());
    let hash = SimdVec::splat(b'#');

    let semicolon_token = SimdVec::splat(SimdTokenType::Semicolon.into());
    let semicolon = SimdVec::splat(b';');

    //

    let mut col_offset = 0;
    text.as_bytes()
        .chunks(SimdVec::LEN)
        .map(SimdVec::load_or_default)
        .enumerate()
        .flat_map(move |(idx, chunk)| {
            let is_new_line = new_line.simd_eq(chunk);
            let is_escape = escape.simd_eq(chunk);
            let is_curly_brace_open = curly_brace_open.simd_eq(chunk);
            let is_curly_brace_close = curly_brace_close.simd_eq(chunk);
            let is_square_bracket_open = square_bracket_open.simd_eq(chunk);
            let is_square_bracket_close = square_bracket_close.simd_eq(chunk);
            let is_round_bracket_open = round_bracket_open.simd_eq(chunk);
            let is_round_bracket_close = round_bracket_close.simd_eq(chunk);
            let is_single_quote = single_quote.simd_eq(chunk);
            let is_double_quote = double_quote.simd_eq(chunk);
            let is_backtick = backtick.simd_eq(chunk);
            let is_forward_slash = forward_slash.simd_eq(chunk);
            let is_star = star.simd_eq(chunk);
            let is_dash = dash.simd_eq(chunk);
            let is_dollar = dollar.simd_eq(chunk);
            let is_at = at.simd_eq(chunk);
            let is_percent = percent.simd_eq(chunk);
            let is_hash = hash.simd_eq(chunk);
            let is_semicolon = semicolon.simd_eq(chunk);

            let tokens = none_token
                | is_new_line.select(new_line_token, none_token)
                | is_escape.select(escape_token, none_token)
                | is_curly_brace_open.select(curly_brace_open_token, none_token)
                | is_curly_brace_close.select(curly_brace_close_token, none_token)
                | is_square_bracket_open.select(square_bracket_open_token, none_token)
                | is_square_bracket_close.select(square_bracket_close_token, none_token)
                | is_round_bracket_open.select(round_bracket_open_token, none_token)
                | is_round_bracket_close.select(round_bracket_close_token, none_token)
                | is_single_quote.select(single_quote_token, none_token)
                | is_double_quote.select(double_quote_token, none_token)
                | is_backtick.select(backtick_token, none_token)
                | is_forward_slash.select(forward_slash_token, none_token)
                | is_star.select(star_token, none_token)
                | is_dash.select(dash_token, none_token)
                | is_dollar.select(dollar_token, none_token)
                | is_at.select(at_token, none_token)
                | is_percent.select(percent_token, none_token)
                | is_hash.select(hash_token, none_token)
                | is_semicolon.select(semicolon_token, none_token);

            // Apply parsed tokens
            let chunk_col = idx * SimdVec::LEN;
            tokens
                .to_array()
                .into_iter()
                .enumerate()
                .flat_map(move |(idx_in_chunk, token)| match token.into() {
                    SimdTokenType::None => None,
                    SimdTokenType::NewLine => {
                        col_offset += idx_in_chunk + 1;

                        return Some(SimdToken {
                            token: token.into(),
                            col: 0,
                        });
                    }
                    token => Some(SimdToken {
                        token: token.into(),
                        col: chunk_col + idx_in_chunk - col_offset,
                    }),
                })
        })
}
