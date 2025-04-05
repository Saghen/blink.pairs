use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::token::{Bracket, Colon, Comma, FatArrow};
use syn::{braced, bracketed, Result};
use syn::{parse_macro_input, Ident, LitStr, Token};

struct MatcherDef {
    name: Ident,
    delimiters: Vec<(String, String)>,
    line_comments: Vec<String>,
    block_comments: Vec<(String, String)>,
    strings: Vec<String>,
    chars: Vec<String>,
    block_strings: Vec<(String, String)>,
}

// Parse the incoming macro definition into a MatcherDef struct
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
                    while !section_content.is_empty() {
                        let open = section_content.parse::<LitStr>()?.value();
                        section_content.parse::<FatArrow>()?;
                        let close = section_content.parse::<LitStr>()?.value();
                        block_strings.push((open, close));

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

/// Calculate the maximum number of characters we need to look ahead
/// based on the longest string we need to match
fn calculate_max_lookahead(def: &MatcherDef) -> usize {
    let mut max_len = 0;

    for (open, close) in &def.delimiters {
        max_len = max_len.max(open.len());
        max_len = max_len.max(close.len());
    }

    for comment in &def.line_comments {
        max_len = max_len.max(comment.len());
    }

    for (open, close) in &def.block_comments {
        max_len = max_len.max(open.len());
        max_len = max_len.max(close.len());
    }

    for s in &def.strings {
        max_len = max_len.max(s.len());
    }

    for s in &def.chars {
        // Always need to lookahead 2 extra bytes ahead for single-char strings
        // So we can check for the second `'` in `'{'`
        max_len = max_len.max(s.len() + 2);
    }

    for (open, close) in &def.block_strings {
        max_len = max_len.max(open.len());
        max_len = max_len.max(close.len());
    }

    // Already have the first byte, so subtract 1
    max_len.saturating_sub(1)
}

/// Generates the match header for the given lookahead
///
/// Examples:
///
/// lookahead = 2
/// Generates: (state, token.byte, token_1_byte, token_2_byte)
///
/// lookahead = 0
/// Generates: (state, token.byte)
fn create_match_header(lookahead: usize) -> TokenStream2 {
    let mut pattern_str = "(state, token.byte".to_string();

    for i in 0..lookahead {
        pattern_str.push_str(&format!(", token_{}_byte", i + 1));
    }

    pattern_str.push(')');

    pattern_str.parse().unwrap()
}

/// Helper function to create a match arm with the appropriate number of wildcards
///
/// Examples:
///
/// max_lookahead = 2
/// pattern = "{"
///
/// Generates: (#state, b'{', _, _) if #cond => { #body }
///
/// max_lookahead = 3
/// pattern = "//"
///
/// Generates: (#state, b'/', b'/', _, _) if #cond => { #body }
fn create_match_arm(
    lookahead: usize,
    state_pattern: TokenStream2,
    pattern: &str,
    condition: Option<TokenStream2>,
    body: TokenStream2,
) -> TokenStream2 {
    // Start with state and main byte
    let mut pattern_str = format!("({}", state_pattern);

    // Add pattern bytes
    for byte in pattern.as_bytes().iter() {
        pattern_str.push_str(&format!(", {}", byte));
    }

    // Add `_` for each lookahead token we didn't use
    for _ in 0..(lookahead - (pattern.len() - 1)) {
        // Add the token byte
        pattern_str.push_str(", _");
    }

    // Close the pattern
    pattern_str.push(')');

    let match_arm: TokenStream2 = pattern_str.parse().unwrap();

    // Combine the pattern with condition and body
    let arm = if let Some(cond) = condition {
        quote! { #match_arm if #cond => { #body } }
    } else {
        quote! { #match_arm => { #body } }
    };

    arm
}

/// Generates an if statement that enforces adjacent tokens
///
/// Examples:
///
/// lookahead = 2
/// Generates: Some(if token_1_distance == 1 && token_2_distance == 2)
///
/// lookahead = 0
/// Generates: None
fn generate_if_adjacent(lookahead: usize) -> Option<TokenStream2> {
    if lookahead == 0 {
        return None;
    }

    let mut pattern_str = "token_1_distance == 1".to_string();

    for i in 1..lookahead {
        let idx = i + 1;
        pattern_str.push_str(&format!(" && token_{}_distance == {}", idx, idx));
    }

    Some(pattern_str.parse().unwrap())
}

/// Helper function to generate lookahead token extractors
fn generate_lookahead_extractors(max_lookahead: usize) -> TokenStream2 {
    let mut extractors = TokenStream2::new();

    for i in 0..max_lookahead {
        let idx = i + 1;
        let token_name = format_ident!("token_{}_byte", idx);
        let distance_name = format_ident!("token_{}_distance", idx);

        let extractor = quote! {
            let (#token_name, #distance_name) = {
                let next_token = tokens
                    .peek()
                    .and_then(|t| if t.byte == b'\n' || t.col != token.col + #idx { None } else { Some(t) });
                (
                    next_token.map(|t| t.byte).unwrap_or(0),
                    (next_token.map(|t| t.col).unwrap_or(usize::MAX) - token.col) as u8,
                )
            };
        };

        extractors.extend(extractor);
    }

    extractors
}

// Helper function to collect all unique tokens
fn collect_tokens(def: &MatcherDef) -> Vec<u8> {
    let mut all_tokens = HashSet::new();

    // Add all token bytes
    for (open, close) in &def.delimiters {
        for c in open.bytes() {
            all_tokens.insert(c);
        }
        for c in close.bytes() {
            all_tokens.insert(c);
        }
    }

    for comment in &def.line_comments {
        for c in comment.bytes() {
            all_tokens.insert(c);
        }
    }

    for (open, close) in &def.block_comments {
        for c in open.bytes() {
            all_tokens.insert(c);
        }
        for c in close.bytes() {
            all_tokens.insert(c);
        }
    }

    for s in &def.strings {
        for c in s.bytes() {
            all_tokens.insert(c);
        }
    }

    for s in &def.chars {
        for c in s.bytes() {
            all_tokens.insert(c);
        }
    }

    // Convert to sorted vector
    let mut tokens_vec: Vec<u8> = all_tokens.into_iter().collect();
    tokens_vec.sort();
    tokens_vec
}

#[proc_macro]
pub fn define_matcher(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as MatcherDef);
    let max_lookahead = calculate_max_lookahead(&def);
    let all_tokens = collect_tokens(&def);
    let token_literals = all_tokens.iter().map(|&t| quote! { #t });
    let lookahead_extractors = generate_lookahead_extractors(max_lookahead);

    // Generate match arms for all patterns
    let mut match_arms = Vec::new();

    // 1. Delimiter patterns
    for (open, close) in &def.delimiters {
        let close_byte = close.as_bytes()[0];

        // Opening delimiter
        let open_arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            open,
            generate_if_adjacent(open.len() - 1),
            quote! {
                matches.push(token.into_match(Some(stack.len())));
                stack.push(#close_byte);
                State::Normal
            },
        );
        match_arms.push(open_arm);

        // Closing delimiter
        let close_arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            close,
            generate_if_adjacent(close.len() - 1),
            quote! {
                if let Some(closing) = stack.last() {
                    if token.byte == *closing {
                        stack.pop();
                    }
                }
                matches.push(token.into_match(Some(stack.len())));
                State::Normal
            },
        );
        match_arms.push(close_arm);
    }

    // 2. Line comment patterns
    for comment in &def.line_comments {
        let arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            comment,
            generate_if_adjacent(comment.len() - 1),
            quote! {
                matches.push(Match::new(MatchToken::LineComment(#comment), token.col));
                tokens.next(); // Skip next token
                State::InLineComment
            },
        );
        match_arms.push(arm);
    }

    // 3. Block comment patterns
    for (open, close) in &def.block_comments {
        let open_arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            open,
            generate_if_adjacent(open.len() - 1),
            quote! {
                matches.push(Match::new(
                    MatchToken::BlockCommentOpen(#open, #close),
                    token.col,
                ));
                tokens.next(); // Skip next token
                State::InBlockComment(#open)
            },
        );
        match_arms.push(open_arm);

        let close_arm = create_match_arm(
            max_lookahead,
            quote! { State::InBlockComment(#open) },
            close,
            generate_if_adjacent(close.len() - 1),
            quote! {
                matches.push(Match::new(
                    MatchToken::BlockCommentClose(#open, #close),
                    token.col,
                ));
                tokens.next(); // Skip next token
                State::Normal
            },
        );
        match_arms.push(close_arm);
    }

    // 4. String patterns
    for delim in &def.strings {
        // Opening string
        let open_arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            delim,
            None,
            quote! {
                matches.push(Match::new(MatchToken::StringOpen(#delim), token.col));
                State::InString(#delim)
            },
        );
        match_arms.push(open_arm);

        // Closing string
        let close_arm = create_match_arm(
            max_lookahead,
            quote! { State::InString(delim) },
            delim,
            Some(quote! { delim == #delim }),
            quote! {
                matches.push(Match::new(MatchToken::StringClose(delim), token.col));
                State::Normal
            },
        );
        match_arms.push(close_arm);
    }

    // 5. Character literal patterns
    for delim in &def.chars {
        let delim_byte = delim.as_bytes()[0];

        let arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            delim,
            Some(
                quote! { token_1_byte == #delim_byte && (token_1_distance == 1 || token_1_distance == 2 ) || token_2_distance == 2 },
            ),
            quote! {
                matches.push(Match::new(MatchToken::StringOpen(#delim), token.col));
                matches.push(Match::new(MatchToken::StringClose(#delim), token.col + 2));
                tokens.next(); // Skip next token
                State::Normal
            },
        );
        match_arms.push(arm);
    }

    // 6. Block string patterns
    for (open, close) in &def.block_strings {
        let open_arm = create_match_arm(
            max_lookahead,
            quote! { State::Normal },
            open,
            generate_if_adjacent(open.len() - 1),
            quote! {
                matches.push(Match::new(
                    MatchToken::BlockStringOpen(#open, #close),
                    token.col,
                ));
                tokens.next(); // Skip next token
                State::InBlockString(#open)
            },
        );
        match_arms.push(open_arm);

        let close_arm = create_match_arm(
            max_lookahead,
            quote! { State::InBlockString(#open) },
            close,
            generate_if_adjacent(close.len() - 1),
            quote! {
                matches.push(Match::new(
                    MatchToken::BlockStringClose(#open, #close),
                    token.col,
                ));
                tokens.next(); // Skip next token
                State::Normal
            },
        );
        match_arms.push(close_arm);
    }

    // Add fallback pattern
    let fallback_arm = quote! { _ => state };
    match_arms.push(fallback_arm);

    // Generate the match statement
    let match_header = create_match_header(max_lookahead);
    let match_stmt = quote! {
        match #match_header {
            #(#match_arms),*
        }
    };

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
                // Generate lookahead tokens based on the calculated max lookahead
                #lookahead_extractors

                #match_stmt
            }
        }
    };

    expanded.into()
}
