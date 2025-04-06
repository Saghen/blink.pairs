#[derive(Debug, Clone, PartialEq)]
pub enum MatchToken {
    DelimiterOpen(&'static str, &'static str),
    DelimiterClose(&'static str, &'static str),

    StringOpen(&'static str),
    StringClose(&'static str),
    BlockStringOpen(&'static str, &'static str),
    BlockStringClose(&'static str, &'static str),

    LineComment(&'static str),
    BlockCommentOpen(&'static str, &'static str),
    BlockCommentClose(&'static str, &'static str),
}

impl MatchToken {
    pub fn text(&self) -> &'static str {
        match self {
            MatchToken::DelimiterOpen(text, _) => text,
            MatchToken::DelimiterClose(_, text) => text,
            MatchToken::StringOpen(text) => text,
            MatchToken::StringClose(text) => text,
            MatchToken::BlockStringOpen(text, _) => text,
            MatchToken::BlockStringClose(_, text) => text,
            MatchToken::LineComment(text) => text,
            MatchToken::BlockCommentOpen(text, _) => text,
            MatchToken::BlockCommentClose(_, text) => text,
        }
    }

    pub fn opening(&self) -> &'static str {
        match self {
            MatchToken::DelimiterOpen(text, _) => text,
            MatchToken::DelimiterClose(_, text) => text,
            MatchToken::StringOpen(text) => text,
            MatchToken::StringClose(text) => text,
            MatchToken::BlockStringOpen(text, _) => text,
            MatchToken::BlockStringClose(text, _) => text,
            MatchToken::LineComment(text) => text,
            MatchToken::BlockCommentOpen(text, _) => text,
            MatchToken::BlockCommentClose(text, _) => text,
        }
    }

    pub fn closing(&self) -> Option<&'static str> {
        match self {
            MatchToken::DelimiterOpen(_, text) => Some(text),
            MatchToken::DelimiterClose(_, text) => Some(text),
            MatchToken::StringOpen(_) => None,
            MatchToken::StringClose(_) => None,
            MatchToken::BlockStringOpen(_, text) => Some(text),
            MatchToken::BlockStringClose(_, text) => Some(text),
            MatchToken::LineComment(_) => None,
            MatchToken::BlockCommentOpen(_, text) => Some(text),
            MatchToken::BlockCommentClose(_, text) => Some(text),
        }
    }

    pub fn is_pair(&self, other: &Self) -> bool {
        use MatchToken::*;
        match (self, other) {
            (DelimiterOpen(self_open, self_close), DelimiterClose(other_open, other_close))
                if self_open == other_open && self_close == other_close =>
            {
                true
            }
            (DelimiterClose(self_open, self_close), DelimiterOpen(other_open, other_close))
                if self_open == other_open && self_close == other_close =>
            {
                true
            }

            (StringOpen(self_open), StringClose(other_open)) if self_open == other_open => true,
            (StringClose(self_open), StringOpen(other_open)) if self_open == other_open => true,

            (
                MatchToken::BlockStringOpen(self_open, self_close),
                MatchToken::BlockStringClose(other_open, other_close),
            ) if self_open == other_open && self_close == other_close => true,
            (
                MatchToken::BlockStringClose(self_open, self_close),
                MatchToken::BlockStringOpen(other_open, other_close),
            ) if self_open == other_open && self_close == other_close => true,

            (
                BlockCommentOpen(self_open, self_close),
                BlockCommentClose(other_open, other_close),
            ) if self_open == other_open && self_close == other_close => true,
            (
                BlockCommentClose(self_open, self_close),
                BlockCommentOpen(other_open, other_close),
            ) if self_open == other_open && self_close == other_close => true,

            _ => false,
        }
    }

    pub fn is_opening(&self) -> bool {
        use MatchToken::*;
        match self {
            DelimiterOpen(_, _) => true,
            StringOpen(_) => true,
            BlockStringOpen(_, _) => true,
            LineComment(_) => true,
            BlockCommentOpen(_, _) => true,
            _ => false,
        }
    }

    pub fn is_closing(&self) -> bool {
        use MatchToken::*;
        match self {
            DelimiterClose(_, _) => true,
            StringClose(_) => true,
            BlockStringClose(_, _) => true,
            LineComment(_) => true,
            BlockCommentClose(_, _) => true,
            _ => false,
        }
    }
}

impl From<u8> for MatchToken {
    fn from(byte: u8) -> Self {
        match byte {
            b'{' => MatchToken::DelimiterOpen("{", "}"),
            b'}' => MatchToken::DelimiterClose("{", "}"),
            b'[' => MatchToken::DelimiterOpen("[", "]"),
            b']' => MatchToken::DelimiterClose("[", "]"),
            b'(' => MatchToken::DelimiterOpen("(", ")"),
            b')' => MatchToken::DelimiterClose("(", ")"),

            _ => panic!("Invalid or ambiguous token type"),
        }
    }
}
