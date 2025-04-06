// TODO: rework with variants that make more sense for usage
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Kind {
    Opening,
    Closing,
    NonPair,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Delimiter(&'static str, &'static str),

    String(&'static str),
    BlockString(&'static str, &'static str),

    LineComment(&'static str),
    BlockComment(&'static str, &'static str),
}

impl Token {
    pub fn opening(&self) -> &'static str {
        match self {
            Token::Delimiter(open, _) => *open,
            Token::String(open) => *open,
            Token::BlockString(open, _) => *open,
            Token::LineComment(open) => *open,
            Token::BlockComment(open, _) => *open,
        }
    }

    pub fn closing(&self) -> Option<&'static str> {
        match self {
            Token::Delimiter(_, close) => Some(*close),
            Token::String(_) => None,
            Token::BlockString(_, close) => Some(*close),
            Token::LineComment(_) => None,
            Token::BlockComment(_, close) => Some(*close),
        }
    }
}

impl From<u8> for Token {
    fn from(byte: u8) -> Self {
        match byte {
            // b'(' | b')' => Token::Delimiter("(", ")"),
            // b'[' | b']' => Token::Delimiter("[", "]"),
            b'{' | b'}' => Token::Delimiter("{", "}"),

            _ => panic!("Invalid or ambiguous token type"),
        }
    }
}
