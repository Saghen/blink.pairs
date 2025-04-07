use std::{cell::Cell, rc::Rc, simd::cmp::SimdPartialEq};

use super::SimdVec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharPos {
    pub byte: u8,
    pub col: usize,
}

impl CharPos {
    pub fn new(byte: u8, col: usize) -> Self {
        Self { byte, col }
    }
}

/// Takes input text and uses SIMD to find the provided list of tokens in the text
/// returning the byte and column position of each token. You can get the row by counting
/// every incoming `\n` token
pub fn tokenize<'s, 't>(text: &'s str, tokens: &'t [u8]) -> impl Iterator<Item = CharPos> + 's {
    let tokens_to_find = tokens
        .iter()
        .flat_map(|&c| {
            match c {
                // Enabled by default, ignore
                0 | b'\n' | b'\\' => None,

                _ => Some(c),
            }
        })
        .collect::<Box<[_]>>();

    //

    // TODO: must use Rc and Cell here since we need to mutate the value inside a closure
    // which uses `move`, so otherwise we would copy, and the value would be reset on every
    // chunk
    let col_offset = Rc::new(Cell::new(0));
    text.as_bytes()
        .chunks(SimdVec::LEN)
        .map(SimdVec::load_or_default)
        .enumerate()
        .flat_map(move |(chunk_idx, chunk)| {
            let mut tokens = SimdVec::splat(0);
            tokens = tokens
                | SimdVec::splat(b'\n')
                    .simd_eq(chunk)
                    .select(SimdVec::splat(b'\n'), SimdVec::splat(0));
            tokens = tokens
                | SimdVec::splat(b'\\')
                    .simd_eq(chunk)
                    .select(SimdVec::splat(b'\\'), SimdVec::splat(0));

            for &char in tokens_to_find.iter() {
                tokens = tokens
                    | SimdVec::splat(char)
                        .simd_eq(chunk)
                        .select(SimdVec::splat(char), SimdVec::splat(0));
            }

            // Apply parsed tokens
            let chunk_col = chunk_idx * SimdVec::LEN;
            let col_offset = col_offset.clone();
            tokens
                .to_array()
                .into_iter()
                .enumerate()
                .flat_map(move |(idx_in_chunk, byte)| match byte {
                    0 => None,
                    b'\n' => {
                        col_offset.set(chunk_col + idx_in_chunk + 1);

                        return Some(CharPos {
                            byte: b'\n',
                            col: 0,
                        });
                    }
                    byte => Some(CharPos {
                        byte,
                        col: chunk_col + idx_in_chunk - col_offset.get(),
                    }),
                })
        })
}

// TODO: come up with a better way to do testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = vec![
            "use crate::r#const::*;",
            "use std::ops::Not;",
            "use std::simd::cmp::*;",
            "use std::simd::num::SimdUint;",
            "use std::simd::{Mask, Simd};",
        ]
        .join("\n");

        assert_eq!(
            tokenize(&text, &[b'(', b')', b'{', b'}']).collect::<Vec<_>>(),
            vec![
                CharPos::new(b'\n', 0),
                CharPos::new(b'\n', 0),
                CharPos::new(b'\n', 0),
                CharPos::new(b'\n', 0),
                CharPos::new(b'{', 15),
                CharPos::new(b'}', 26),
            ]
        );
    }
}
