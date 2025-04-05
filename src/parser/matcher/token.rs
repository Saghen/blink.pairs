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

    InlineSpan(&'static str, &'static str, &'static str),
    BlockSpan(&'static str, &'static str, &'static str),
}

impl Token {
    pub fn opening(&self) -> &'static str {
        match self {
            Token::Delimiter(open, _) => *open,
            Token::String(open) => *open,
            Token::BlockString(open, _) => *open,
            Token::LineComment(open) => *open,
            Token::BlockComment(open, _) => *open,
            Token::InlineSpan(_, open, _) => *open,
            Token::BlockSpan(_, open, _) => *open,
        }
    }

    pub fn closing(&self) -> Option<&'static str> {
        match self {
            Token::Delimiter(_, close) => Some(*close),
            Token::String(_) => None,
            Token::BlockString(_, close) => Some(*close),
            Token::LineComment(_) => None,
            Token::BlockComment(_, close) => Some(*close),
            Token::InlineSpan(_, _, close) => Some(*close),
            Token::BlockSpan(_, _, close) => Some(*close),
        }
    }
}
