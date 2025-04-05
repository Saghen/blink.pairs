use std::simd::cmp::SimdPartialEq;

use super::{Match, SimdVec};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenPos {
    pub byte: u8,
    pub col: usize,
}

impl TokenPos {
    pub fn into_match(self, stack_height: Option<usize>) -> Match {
        Match {
            token: self.byte.into(),
            col: self.col,
            stack_height,
        }
    }
}

/// Takes input text and uses SIMD to find the provided list of tokens in the text
/// returning the byte and column position of each token. You can get the row by counting
/// every incoming `\n` token
pub fn tokenize(text: &str, tokens: Vec<u8>) -> impl Iterator<Item = TokenPos> + '_ {
    // Tokens
    let none = SimdVec::splat(0);

    let new_line = SimdVec::splat(b'\n');
    let escape = SimdVec::splat(b'\\');

    let tokens_to_find = tokens
        .into_iter()
        .flat_map(|c| {
            match c {
                // Enabled by default, ignore
                b'\n' | b'\\' => None,

                _ => Some(SimdVec::splat(c)),
            }
        })
        .collect::<Vec<_>>();

    //

    let mut col_offset = 0;
    text.as_bytes()
        .chunks(SimdVec::LEN)
        .map(SimdVec::load_or_default)
        .enumerate()
        .flat_map(move |(idx, chunk)| {
            let mut tokens = none;
            tokens = tokens | new_line.simd_eq(chunk).select(new_line, none);
            tokens = tokens | escape.simd_eq(chunk).select(escape, none);

            for char in tokens_to_find.iter() {
                tokens = tokens | char.simd_eq(chunk).select(*char, none);
            }

            // Apply parsed tokens
            let chunk_col = idx * SimdVec::LEN;
            tokens
                .to_array()
                .into_iter()
                .enumerate()
                .flat_map(move |(idx_in_chunk, byte)| match byte {
                    0 => None,
                    b'\n' => {
                        col_offset += idx_in_chunk + 1;

                        return Some(TokenPos {
                            byte: b'\n',
                            col: 0,
                        });
                    }
                    byte => Some(TokenPos {
                        byte,
                        col: chunk_col + idx_in_chunk - col_offset,
                    }),
                })
        })
}
