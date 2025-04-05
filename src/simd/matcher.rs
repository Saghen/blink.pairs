use itertools::MultiPeek;

use super::{State, Token, TokenPos};

pub trait Matcher {
    fn tokens(&self) -> Vec<Token>;

    fn call<I>(
        &mut self,
        matches: &mut Vec<Match>,
        stack: &mut Vec<Token>,
        tokens: &mut MultiPeek<I>,
        state: State,
        token: TokenPos,
    ) -> State
    where
        I: Iterator<Item = TokenPos>;
}

#[derive(Debug, PartialEq)]
pub struct Match {
    pub token: MatchToken,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl Match {
    pub fn new(token: MatchToken, col: usize) -> Self {
        Self {
            token,
            col,
            stack_height: None,
        }
    }
}

impl From<TokenPos> for Match {
    fn from(token: TokenPos) -> Self {
        Match {
            token: token.token.into(),
            col: token.col,
            stack_height: None,
        }
    }
}

#[derive(Debug, PartialEq)]
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

impl From<Token> for MatchToken {
    fn from(token: Token) -> Self {
        match token {
            Token::CurlyBraceOpen => MatchToken::DelimiterOpen("{", "}"),
            Token::CurlyBraceClose => MatchToken::DelimiterClose("{", "}"),
            Token::SquareBracketOpen => MatchToken::DelimiterOpen("[", "]"),
            Token::SquareBracketClose => MatchToken::DelimiterClose("[", "]"),
            Token::ParenthesisOpen => MatchToken::DelimiterOpen("(", ")"),
            Token::ParenthesisClose => MatchToken::DelimiterClose("(", ")"),

            _ => panic!("Invalid or ambiguous token type"),
        }
    }
}
