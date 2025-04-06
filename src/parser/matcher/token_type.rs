use super::MatchToken;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MatchTokenType {
    Delimiter = 0,
    String = 1,
    BlockString = 2,
    LineComment = 3,
    BlockComment = 4,
}

impl MatchTokenType {
    pub fn matches(&self, token: &MatchToken) -> bool {
        use MatchTokenType::*;
        match (self, token) {
            (Delimiter, MatchToken::DelimiterOpen(_, _))
            | (Delimiter, MatchToken::DelimiterClose(_, _))
            | (String, MatchToken::StringOpen(_))
            | (String, MatchToken::StringClose(_))
            | (BlockString, MatchToken::BlockStringOpen(_, _))
            | (BlockString, MatchToken::BlockStringClose(_, _))
            | (LineComment, MatchToken::LineComment(_))
            | (BlockComment, MatchToken::BlockCommentOpen(_, _))
            | (BlockComment, MatchToken::BlockCommentClose(_, _)) => true,

            _ => false,
        }
    }
}

impl TryFrom<u8> for MatchTokenType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MatchTokenType::Delimiter),
            1 => Ok(MatchTokenType::String),
            2 => Ok(MatchTokenType::BlockString),
            3 => Ok(MatchTokenType::LineComment),
            4 => Ok(MatchTokenType::BlockComment),
            _ => Err(()),
        }
    }
}
