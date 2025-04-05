use itertools::MultiPeek;

use crate::simd::{
    Match, MatchToken, Matcher,
    State::{self, *},
    Token::{self, *},
    TokenPos,
};

pub struct C;

impl Matcher for C {
    fn tokens(&self) -> Vec<Token> {
        vec![
            CurlyBraceOpen,
            CurlyBraceClose,
            SquareBracketOpen,
            SquareBracketClose,
            ParenthesisOpen,
            ParenthesisClose,
            SingleQuote,
            DoubleQuote,
            ForwardSlash,
            Star,
        ]
    }

    fn call<I>(
        &mut self,
        matches: &mut Vec<Match>,
        stack: &mut Vec<Token>,
        tokens: &mut MultiPeek<I>,
        state: State,
        token: TokenPos,
    ) -> State
    where
        I: Iterator<Item = TokenPos>,
    {
        let next_token = tokens.peek();

        let col = token.col;
        match (
            state,
            token.token,
            next_token.map(|t| t.token).unwrap_or(None),
        ) {
            // Delimiters
            (Normal, CurlyBraceOpen | SquareBracketOpen | ParenthesisOpen, _) => {
                matches.push(token.into_match(Some(stack.len())));
                stack.push(token.token.get_closing().unwrap_or(token.token));
                Normal
            }
            (Normal, CurlyBraceClose | SquareBracketClose | ParenthesisClose, _) => {
                if let Some(closing) = stack.last() {
                    if token.token == *closing {
                        stack.pop();
                    }
                }

                matches.push(token.into_match(Some(stack.len())));
                Normal
            }

            // Strings
            (Normal, SingleQuote, SingleQuote) => {
                // TODO: doesn't work if there's a token inside the single-char string
                let next_token_col = next_token.map(|t| t.col).unwrap_or(usize::MAX);
                if next_token_col == col + 2 {
                    matches.push(Match::new(MatchToken::StringOpen("'"), col));
                    matches.push(Match::new(MatchToken::StringClose("'"), col + 2));
                    tokens.next(); // Skip next token
                }
                Normal
            }

            (Normal, DoubleQuote, _) => {
                matches.push(Match::new(MatchToken::StringOpen("\""), col));
                InString("\"")
            }
            (InString(delim), DoubleQuote, _) if delim == "\"" => {
                matches.push(Match::new(MatchToken::StringClose(delim), col));
                Normal
            }

            // Line comments
            (Normal, ForwardSlash, ForwardSlash) => {
                matches.push(Match::new(MatchToken::LineComment("//"), col));
                tokens.next(); // Skip next token
                InLineComment
            }

            // Block comments
            (Normal, ForwardSlash, Star) => {
                matches.push(Match::new(MatchToken::BlockCommentOpen("/*", "*/"), col));
                tokens.next(); // Skip next token
                InBlockComment("/*")
            }
            (InBlockComment("/*"), Star, ForwardSlash) => {
                matches.push(Match::new(MatchToken::BlockCommentClose("/*", "*/"), col));
                tokens.next(); // Skip next token
                Normal
            }

            _ => state,
        }
    }
}
