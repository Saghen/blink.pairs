use nom::{
    branch::alt,
    bytes::complete::tag,
    character::anychar,
    combinator::{not, value},
    multi::{many0, many_till},
    sequence::preceded,
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CToken<'a> {
    DelimiterOpen((&'a str, &'a str)),
    DelimiterClose(&'a str),
    LineComment,
    BlockCommentOpen((&'a str, &'a str)),
    BlockCommentClose(&'a str),
    String,
    BlockStringOpen((&'a str, &'a str)),
    BlockStringClose(&'a str),
    BlockStringSymmetric(&'a str),
    Escape,
    NonToken,
}

fn token(input: &str) -> IResult<&str, CToken> {
    alt((
        // DelimiterOpen
        value(CToken::DelimiterOpen(("(", ")")), tag("(")),
        value(CToken::DelimiterOpen(("[", "]")), tag("[")),
        value(CToken::DelimiterOpen(("{", "}")), tag("{")),
        // DelimiterClose
        value(CToken::DelimiterClose(")"), tag(")")),
        value(CToken::DelimiterClose("]"), tag("]")),
        value(CToken::DelimiterClose("}"), tag("}")),
        // Comments
        value(CToken::LineComment, tag("//")),
        value(CToken::BlockCommentOpen(("/*", "*/")), tag("/*")),
        value(CToken::BlockCommentClose("*/"), tag("*/")),
        // Escape
        value(CToken::Escape, tag("\\")),
    ))
    .parse(input)
}

// Main parser that collects tokens and skips non-tokens
pub fn parse(input: &str) -> IResult<&str, Vec<CToken>> {
    let mut result = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        // Try to parse a token
        match token(remaining) {
            Ok((rest, token)) => {
                result.push(token);
                remaining = rest;
            }
            // If no token, skip one character
            Err(_) => {
                // Safe because we checked that remaining is not empty
                let (rest, _) = anychar(remaining)?;
                remaining = rest;
            }
        }
    }

    Ok(("", result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse(" {"),
            Ok(("", vec![CToken::DelimiterOpen(("{", "}"))]))
        );

        assert_eq!(
            parse("int main(int argc, char **argv) { return 0; }"),
            Ok((
                "",
                vec![
                    CToken::DelimiterOpen(("(", ")")),
                    CToken::DelimiterClose(")"),
                    CToken::DelimiterOpen(("{", "}")),
                    CToken::DelimiterClose("}"),
                ]
            ))
        );
    }
}
