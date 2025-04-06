use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Comma, FatArrow};
use syn::{braced, bracketed, Result};
use syn::{Ident, LitStr};

use std::collections::HashSet;

pub struct MatcherDef {
    pub name: Ident,
    pub delimiters: Vec<(String, String)>,
    pub line_comments: Vec<String>,
    pub block_comments: Vec<(String, String)>,
    pub strings: Vec<String>,
    pub chars: Vec<String>,
    pub block_strings: Vec<(String, String)>,
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
                        assert_eq!(open.len(), 1, "Delimiter must be a single character");
                        section_content.parse::<FatArrow>()?;
                        let close = section_content.parse::<LitStr>()?.value();
                        assert_eq!(close.len(), 1, "Delimiter must be a single character");
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
                        let delim = section_content.parse::<LitStr>()?.value();
                        assert_eq!(delim.len(), 1, "Delimiter must be a single character");
                        chars.push(delim);
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

// Helper function to collect all unique tokens
pub fn collect_tokens(def: &MatcherDef) -> Vec<u8> {
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

    for (open, close) in &def.block_strings {
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
