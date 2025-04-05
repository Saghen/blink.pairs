use std::simd::cmp::SimdPartialEq;

use super::{Match, SimdVec};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenPos {
    pub token: Token,
    pub col: usize,
}

impl TokenPos {
    pub fn into_match(self, stack_height: Option<usize>) -> Match {
        Match {
            token: self.token.into(),
            col: self.col,
            stack_height,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
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

impl From<Token> for u8 {
    fn from(val: Token) -> Self {
        val as u8
    }
}

impl From<u8> for Token {
    fn from(val: u8) -> Self {
        match val {
            0 => Token::None,
            1 => Token::NewLine,
            2 => Token::Escape,
            3 => Token::CurlyBraceOpen,
            4 => Token::CurlyBraceClose,
            5 => Token::SquareBracketOpen,
            6 => Token::SquareBracketClose,
            7 => Token::ParenthesisOpen,
            8 => Token::ParenthesisClose,
            9 => Token::SingleQuote,
            10 => Token::DoubleQuote,
            11 => Token::BackTick,
            12 => Token::ForwardSlash,
            13 => Token::Star,
            14 => Token::Dash,
            15 => Token::Dollar,
            16 => Token::At,
            17 => Token::Percent,
            18 => Token::Hash,
            19 => Token::Semicolon,
            _ => panic!("Invalid token value: {}", val),
        }
    }
}

impl Token {
    pub fn char(self) -> u8 {
        match self {
            Token::None => 0,
            Token::NewLine => b'\n',
            Token::Escape => b'\\',
            Token::CurlyBraceOpen => b'{',
            Token::CurlyBraceClose => b'}',
            Token::SquareBracketOpen => b'[',
            Token::SquareBracketClose => b']',
            Token::ParenthesisOpen => b'(',
            Token::ParenthesisClose => b')',
            Token::SingleQuote => b'\'',
            Token::DoubleQuote => b'"',
            Token::BackTick => b'`',
            Token::ForwardSlash => b'/',
            Token::Star => b'*',
            Token::Dash => b'-',
            Token::Dollar => b'$',
            Token::At => b'@',
            Token::Percent => b'%',
            Token::Hash => b'#',
            Token::Semicolon => b';',
        }
    }

    pub fn get_closing(&self) -> Option<Self> {
        match self {
            Token::CurlyBraceOpen => Some(Token::CurlyBraceClose),
            Token::SquareBracketOpen => Some(Token::SquareBracketClose),
            Token::ParenthesisOpen => Some(Token::ParenthesisClose),
            _ => None,
        }
    }
}

pub fn tokenize(text: &str, tokens: Vec<Token>) -> impl Iterator<Item = TokenPos> + '_ {
    // Tokens
    let none_token = SimdVec::splat(Token::None.into());

    let new_line_token = SimdVec::splat(Token::NewLine.into());
    let new_line = SimdVec::splat(Token::NewLine.char());

    let escape_token = SimdVec::splat(Token::Escape.into());
    let escape = SimdVec::splat(Token::Escape.char());

    let tokens_to_find = tokens
        .into_iter()
        .flat_map(|t| {
            match t {
                // Enabled by default, ignore
                Token::None | Token::NewLine | Token::Escape => None,

                _ => Some((SimdVec::splat(t.into()), SimdVec::splat(t.char()))),
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
            let mut tokens = none_token;
            tokens = tokens | new_line.simd_eq(chunk).select(new_line_token, none_token);
            tokens = tokens | escape.simd_eq(chunk).select(escape_token, none_token);

            for (token, char) in tokens_to_find.iter() {
                tokens = tokens | char.simd_eq(chunk).select(*token, none_token);
            }

            // Apply parsed tokens
            let chunk_col = idx * SimdVec::LEN;
            tokens
                .to_array()
                .into_iter()
                .enumerate()
                .flat_map(move |(idx_in_chunk, token)| match token.into() {
                    Token::None => None,
                    Token::NewLine => {
                        col_offset += idx_in_chunk + 1;

                        return Some(TokenPos {
                            token: token.into(),
                            col: 0,
                        });
                    }
                    token => Some(TokenPos {
                        token: token.into(),
                        col: chunk_col + idx_in_chunk - col_offset,
                    }),
                })
        })
}
