use itertools::MultiPeek;

use crate::simd::parse::{SimdMatch, SimdMatchType};
use crate::simd::tokenize::{
    SimdToken,
    SimdTokenType::{self, *},
};
use crate::simd::State::{self, *};

pub fn c_matcher(
    matches: &mut Vec<SimdMatch>,
    stack: &mut Vec<SimdTokenType>,
    tokens: &mut MultiPeek<impl Iterator<Item = SimdToken>>,
    state: State,
    token: SimdToken,
) -> State {
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
            stack.push(token.token);
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
                matches.push(SimdMatch::new(SimdMatchType::StringOpen("'"), col));
                matches.push(SimdMatch::new(SimdMatchType::StringClose("'"), col + 2));
                tokens.next(); // Skip next token
            }
            Normal
        }

        (Normal, DoubleQuote, _) => {
            matches.push(SimdMatch::new(SimdMatchType::StringOpen("\""), col));
            InString("\"")
        }
        (InString(delim), DoubleQuote, _) if delim == "\"" => {
            matches.push(SimdMatch::new(SimdMatchType::StringClose(delim), col));
            Normal
        }

        // Line comments
        (Normal, ForwardSlash, ForwardSlash) => {
            matches.push(SimdMatch::new(SimdMatchType::LineComment("//"), col));
            tokens.next(); // Skip next token
            InLineComment
        }

        // Block comments
        (Normal, ForwardSlash, Star) => {
            matches.push(SimdMatch::new(
                SimdMatchType::BlockCommentOpen("/*", "*/"),
                col,
            ));
            tokens.next(); // Skip next token
            InBlockComment("/*")
        }
        (InBlockComment("/*"), Star, ForwardSlash) => {
            matches.push(SimdMatch::new(
                SimdMatchType::BlockCommentClose("/*", "*/"),
                col,
            ));
            tokens.next(); // Skip next token
            Normal
        }

        _ => state,
    }
}
