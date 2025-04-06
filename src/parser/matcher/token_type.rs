use super::Token;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TokenType {
    Delimiter = 0,
    String = 1,
    BlockString = 2,
    LineComment = 3,
    BlockComment = 4,
}

impl TokenType {
    pub fn matches(&self, token: &Token) -> bool {
        use TokenType::*;
        match (self, token) {
            (Delimiter, Token::Delimiter(_, _))
            | (String, Token::String(_))
            | (BlockString, Token::BlockString(_, _))
            | (LineComment, Token::LineComment(_))
            | (BlockComment, Token::BlockComment(_, _)) => true,

            _ => false,
        }
    }
}

impl TryFrom<u8> for TokenType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenType::Delimiter),
            1 => Ok(TokenType::String),
            2 => Ok(TokenType::BlockString),
            3 => Ok(TokenType::LineComment),
            4 => Ok(TokenType::BlockComment),
            _ => Err(()),
        }
    }
}
