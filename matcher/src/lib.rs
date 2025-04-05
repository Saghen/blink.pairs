use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Comma, FatArrow};
use syn::{braced, bracketed, Result};
use syn::{parse_macro_input, Ident, LitStr};

struct MatcherDef {
    name: Ident,
    delimiters: Vec<(String, String)>,
    line_comments: Vec<String>,
    block_comments: Vec<(String, String)>,
    strings: Vec<String>,
    chars: Vec<String>,
    block_strings: Vec<String>,
}

// Parser implementation for the matcher definition
impl Parse for MatcherDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let content;
        braced!(content in input);

        let mut delimiters = Vec::new();
        let mut line_comments = Vec::new();
        let mut block_comments = Vec::new();
        let mut strings = Vec::new();
        let mut chars = Vec::new();
        let mut block_strings = Vec::new();

        // Parse each section
        while !content.is_empty() {
            let section_name = content.parse::<Ident>()?;
            content.parse::<Colon>()?;

            let section_content;
            bracketed!(section_content in content);

            match section_name.to_string().as_str() {
                "delimiters" => {
                    while !section_content.is_empty() {
                        let open = section_content.parse::<LitStr>()?.value();
                        section_content.parse::<FatArrow>()?;
                        let close = section_content.parse::<LitStr>()?.value();
                        delimiters.push((open, close));

                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                "line_comment" => {
                    while !section_content.is_empty() {
                        line_comments.push(section_content.parse::<LitStr>()?.value());
                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                "block_comment" => {
                    while !section_content.is_empty() {
                        let open = section_content.parse::<LitStr>()?.value();
                        section_content.parse::<FatArrow>()?;
                        let close = section_content.parse::<LitStr>()?.value();
                        block_comments.push((open, close));

                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                "string" => {
                    while !section_content.is_empty() {
                        strings.push(section_content.parse::<LitStr>()?.value());
                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                "char" => {
                    while !section_content.is_empty() {
                        chars.push(section_content.parse::<LitStr>()?.value());
                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                "block_string" => {
                    // Similar to other sections
                    while !section_content.is_empty() {
                        block_strings.push(section_content.parse::<LitStr>()?.value());
                        if !section_content.is_empty() {
                            section_content.parse::<Comma>()?;
                        }
                    }
                }
                _ => return Err(syn::Error::new(section_name.span(), "Unknown section name")),
            }

            if !content.is_empty() {
                content.parse::<Comma>()?;
            }
        }

        Ok(MatcherDef {
            name,
            delimiters,
            line_comments,
            block_comments,
            strings,
            chars,
            block_strings,
        })
    }
}

#[proc_macro]
pub fn define_matcher(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as MatcherDef);

    // Collect all unique tokens for the tokens() method
    let mut all_tokens = Vec::new();

    // Add delimiter tokens
    for (open, close) in &def.delimiters {
        for c in open.chars() {
            all_tokens.push(c as u8);
        }
        for c in close.chars() {
            all_tokens.push(c as u8);
        }
    }

    // Add comment tokens
    for comment in &def.line_comments {
        for c in comment.chars() {
            all_tokens.push(c as u8);
        }
    }

    for (open, close) in &def.block_comments {
        for c in open.chars() {
            all_tokens.push(c as u8);
        }
        for c in close.chars() {
            all_tokens.push(c as u8);
        }
    }

    // Add string tokens
    for s in &def.strings {
        for c in s.chars() {
            all_tokens.push(c as u8);
        }
    }

    for s in &def.chars {
        for c in s.chars() {
            all_tokens.push(c as u8);
        }
    }

    // Deduplicate tokens
    all_tokens.sort();
    all_tokens.dedup();

    // Generate token literals
    let token_literals = all_tokens.iter().map(|&t| quote! { #t });

    // Generate pattern matches for delimiters
    let delimiter_matches = def.delimiters.iter().flat_map(|(open, close)| {
        let open_byte = open.as_bytes()[0];
        let close_byte = close.as_bytes()[0];

        vec![
            quote! {
                (State::Normal, #open_byte, _) => {
                    matches.push(token.into_match(Some(stack.len())));
                    stack.push(#close_byte);
                    State::Normal
                }
            },
            quote! {
                (State::Normal, #close_byte, _) => {
                    if let Some(closing) = stack.last() {
                        if token.byte == *closing {
                            stack.pop();
                        }
                    }
                    matches.push(token.into_match(Some(stack.len())));
                    State::Normal
                }
            },
        ]
    });

    // Generate pattern matches for line comments
    let line_comment_matches = def.line_comments.iter().map(|comment| {
        let start = comment.as_bytes()[0];
        let next = comment.as_bytes()[1];

        quote! {
            (State::Normal, #start, #next) if distance == 1 => {
                matches.push(Match::new(MatchToken::LineComment(#comment), token.col));
                tokens.next(); // Skip next token
                State::InLineComment
            }
        }
    });

    // Generate pattern matches for block comments
    let block_comment_matches = def.block_comments.iter().flat_map(|(open, close)| {
        let open_start = open.as_bytes()[0];
        let open_next = open.as_bytes()[1];
        let close_start = close.as_bytes()[0];
        let close_next = close.as_bytes()[1];

        vec![
            quote! {
                (State::Normal, #open_start, #open_next) if distance == 1 => {
                    matches.push(Match::new(
                        MatchToken::BlockCommentOpen(#open, #close),
                        token.col,
                    ));
                    tokens.next(); // Skip next token
                    State::InBlockComment(#open)
                }
            },
            quote! {
                (State::InBlockComment(#open), #close_start, #close_next) if distance == 1 => {
                    matches.push(Match::new(
                        MatchToken::BlockCommentClose(#open, #close),
                        token.col,
                    ));
                    tokens.next(); // Skip next token
                    State::Normal
                }
            },
        ]
    });

    // Generate pattern matches for strings
    let string_matches = def.strings.iter().flat_map(|delim| {
        let delim_byte = delim.as_bytes()[0];

        vec![
            quote! {
                (State::Normal, #delim_byte, _) => {
                    matches.push(Match::new(MatchToken::StringOpen(#delim), token.col));
                    State::InString(#delim)
                }
            },
            quote! {
                (State::InString(delim), #delim_byte, _) if delim == #delim => {
                    matches.push(Match::new(MatchToken::StringClose(delim), token.col));
                    State::Normal
                }
            },
        ]
    });

    // Generate pattern matches for character literals
    let char_matches = def.chars.iter().map(|delim| {
        let delim_byte = delim.as_bytes()[0];

        quote! {
            (State::Normal, #delim_byte, #delim_byte) if distance <= 2 => {
                let next_token_col = next_token.map(|t| t.col).unwrap_or(usize::MAX);
                if next_token_col == token.col + 2 {
                    matches.push(Match::new(MatchToken::StringOpen(#delim), token.col));
                    matches.push(Match::new(MatchToken::StringClose(#delim), token.col + 2));
                    tokens.next(); // Skip next token
                }
                State::Normal
            }
        }
    });

    let name = &def.name;

    // Generate the full implementation
    let expanded = quote! {
        pub struct #name;

        impl Matcher for #name {
            fn tokens(&self) -> Vec<u8> {
                vec![#(#token_literals),*]
            }

            fn call<I>(
                &mut self,
                matches: &mut Vec<Match>,
                stack: &mut Vec<u8>,
                tokens: &mut MultiPeek<I>,
                state: State,
                token: TokenPos,
            ) -> State
            where
                I: Iterator<Item = TokenPos>,
            {
                let mut next_token = tokens.peek();
                // Ignore if it's the next line
                if let Some(token) = next_token {
                    if token.byte == b'\n' {
                        next_token = None;
                    }
                }

                let distance = next_token.map(|t| t.col).unwrap_or(usize::MAX) - token.col;

                match (state, token.byte, next_token.map(|t| t.byte).unwrap_or(0)) {
                    // Block comment patterns
                    #(#block_comment_matches),*,

                    // Line comment patterns
                    #(#line_comment_matches),*,

                    // String patterns
                    #(#string_matches),*,

                    // Character patterns
                    #(#char_matches),*,

                    // Delimiter patterns
                    #(#delimiter_matches),*,

                    _ => state,
                }
            }
        }
    };

    expanded.into()
}
