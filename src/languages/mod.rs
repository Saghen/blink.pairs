use mlua::{serde::Serializer, IntoLua};
use serde::Serialize;

mod c;
mod clojure;
mod cpp;
mod csharp;
mod dart;
mod elixir;
mod erlang;
mod fsharp;
mod go;
mod haskell;
mod haxe;
mod java;
mod javascript;
mod json;
mod kotlin;
mod latex;
mod lean;
mod lua;
mod markdown;
mod objc;
mod ocaml;
mod perl;
mod php;
mod python;
mod r;
mod ruby;
mod rust;
mod scala;
mod shell;
mod swift;
mod toml;
mod typst;
mod zig;

pub use c::*;
pub use clojure::*;
pub use cpp::*;
pub use csharp::*;
pub use dart::*;
pub use elixir::*;
pub use erlang::*;
pub use fsharp::*;
pub use go::*;
pub use haskell::*;
pub use haxe::*;
pub use java::*;
pub use javascript::*;
pub use json::*;
pub use kotlin::*;
pub use latex::*;
pub use lean::*;
pub use lua::*;
pub use markdown::*;
pub use objc::*;
pub use ocaml::*;
pub use perl::*;
pub use php::*;
pub use python::*;
pub use r::*;
pub use ruby::*;
pub use rust::*;
pub use scala::*;
pub use shell::*;
pub use swift::*;
pub use toml::*;
pub use typst::*;
pub use zig::*;

#[derive(Debug, Clone, Copy)]
pub enum Token {
    DelimiterOpen {
        text: &'static str,
        closing: &'static str,
    },
    DelimiterClose(&'static str),
    LineComment,
    BlockCommentOpen {
        text: &'static str,
        closing: &'static str,
    },
    BlockCommentClose(&'static str),
    String,
    BlockStringOpen {
        text: &'static str,
        closing: &'static str,
    },
    BlockStringClose(&'static str),
    BlockStringSymmetric(&'static str),
    InlineSpanOpen {
        text: &'static str,
        closing: &'static str,
        name: &'static str,
    },
    InlineSpanClose(&'static str),
    InlineSpanSymmetric {
        text: &'static str,
        name: &'static str,
    },
    BlockSpanOpen {
        text: &'static str,
        closing: &'static str,
        name: &'static str,
    },
    BlockSpanClose(&'static str),
    BlockSpanSymmetric {
        text: &'static str,
        name: &'static str,
    },
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash)]
#[repr(u8)]
pub enum TokenType {
    Delimiter = 0,
    String = 1,
    BlockComment = 2,
    InlineSpan = 3,
    BlockSpan = 4,
}

impl TryFrom<u8> for TokenType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenType::Delimiter),
            1 => Ok(TokenType::String),
            2 => Ok(TokenType::BlockComment),
            3 => Ok(TokenType::InlineSpan),
            4 => Ok(TokenType::BlockSpan),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AvailableToken {
    type_: TokenType,
    opening: String,
    closing: String,
    name: Option<String>,
}

impl IntoLua for AvailableToken {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        self.serialize(Serializer::new(lua))
    }
}

// static string newtype - needed because logos callbacks can't directly return `&'static str`
struct SStr(&'static str);

// TODO: Is there a better way to handle the RustToken and rust_tokens identifiers?
#[macro_export]
macro_rules! define_token_enum {
    // New format with inline_span and block_span sections
    ($name:ident, $get_tokens:ident, {
        delimiters: { $($open:literal => $close:literal),* $(,)? },
        line_comment: [ $($line_comment:literal),* $(,)? ],
        block_comment: [ $($block_comment_open:literal => $block_comment_close:literal),* $(,)? ],
        string_regex: [ $($string_regex:literal),* $(,)? ],
        block_string: [
            $(symmetric $block_string_symmetric:literal),*
            $($block_string_open:literal => $block_string_close:literal),* $(,)?
        ],
        inline_span: [
            $(symmetric $inline_span_symmetric:literal => $inline_span_name:literal),*
            $($inline_span_open:literal => $inline_span_close:literal => $inline_span_open_name:literal),* $(,)?
        ],
        block_span: [
            $(symmetric $block_span_symmetric:literal => $block_span_name:literal),*
            $($block_span_open:literal => $block_span_close:literal => $block_span_open_name:literal),* $(,)?
        ]
    }) => {
        #[allow(unused, private_interfaces)] // Ignore warnings about unused variants and SStr interface leakage
        #[derive(logos::Logos)]
        #[logos(skip r"[ \t\f]+")] // Skip whitespace
        #[logos(subpattern dstring = r#""([^"\\]|\\.)*""#)] // " string
        #[logos(subpattern sstring = r#"'([^'\\]|\\.)*'"#)] // ' string
        #[logos(subpattern schar = r#"'([^'\\]|\\.)'"#)] // ' char (single-character)
        pub enum $name {
            $(#[token($open, |_|  {($crate::languages::SStr($open), $crate::languages::SStr($close))} )])*
            DelimiterOpen(($crate::languages::SStr, $crate::languages::SStr)),

            $(#[token($close, |_| $crate::languages::SStr($close) )])*
            DelimiterClose($crate::languages::SStr),

            $(#[token($line_comment)])*
            LineComment,

            $(#[token($block_comment_open, |_| {($crate::languages::SStr($block_comment_open), $crate::languages::SStr($block_comment_close))} )])*
            BlockCommentOpen(($crate::languages::SStr, $crate::languages::SStr)),
            $(#[token($block_comment_close, |_| $crate::languages::SStr($block_comment_close) )])*
            BlockCommentClose($crate::languages::SStr),

            $(#[regex($string_regex)])*
            String,

            $(#[token($block_string_open, |_| {($crate::languages::SStr($block_string_open), $crate::languages::SStr($block_string_close))}, priority = 10 )])*
            BlockStringOpen(($crate::languages::SStr, $crate::languages::SStr)),
            $(#[token($block_string_close, |_| $crate::languages::SStr($block_string_close), priority = 10 )])*
            BlockStringClose($crate::languages::SStr),

            $(#[token($block_string_symmetric, |_| $crate::languages::SStr($block_string_symmetric), priority = 10 )])*
            BlockStringSymmetric($crate::languages::SStr),

            $(#[token($inline_span_open, |_| {($crate::languages::SStr($inline_span_open), $crate::languages::SStr($inline_span_close), $crate::languages::SStr($inline_span_open_name))}, priority = 10 )])*
            InlineSpanOpen(($crate::languages::SStr, $crate::languages::SStr, $crate::languages::SStr)),
            $(#[token($inline_span_close, |_| $crate::languages::SStr($inline_span_close), priority = 10 )])*
            InlineSpanClose($crate::languages::SStr),

            $(#[token($inline_span_symmetric, |_| {($crate::languages::SStr($inline_span_symmetric), $crate::languages::SStr($inline_span_name))}, priority = 10 )])*
            InlineSpanSymmetric(($crate::languages::SStr, $crate::languages::SStr)),

            $(#[token($block_span_open, |_| {($crate::languages::SStr($block_span_open), $crate::languages::SStr($block_span_close), $crate::languages::SStr($block_span_open_name))}, priority = 10 )])*
            BlockSpanOpen(($crate::languages::SStr, $crate::languages::SStr, $crate::languages::SStr)),
            $(#[token($block_span_close, |_| $crate::languages::SStr($block_span_close), priority = 10 )])*
            BlockSpanClose($crate::languages::SStr),

            $(#[token($block_span_symmetric, |_| {($crate::languages::SStr($block_span_symmetric), $crate::languages::SStr($block_span_name))}, priority = 10 )])*
            BlockSpanSymmetric(($crate::languages::SStr, $crate::languages::SStr)),

            #[token("\\")]
            Escape,
        }

        impl From<$name> for $crate::languages::Token {
            fn from(value: $name) -> Self {
                match value {
                    $name::DelimiterOpen((text, closing)) => Self::DelimiterOpen { text: text.0, closing: closing.0 },
                    $name::DelimiterClose(s) => Self::DelimiterClose(s.0),
                    $name::LineComment => Self::LineComment,
                    $name::BlockCommentOpen((text, closing)) => Self::BlockCommentOpen { text: text.0, closing: closing.0 },
                    $name::BlockCommentClose(close) => Self::BlockCommentClose(close.0),
                    $name::String => Self::String,
                    $name::BlockStringOpen((text, closing)) => Self::BlockStringOpen { text: text.0, closing: closing.0 },
                    $name::BlockStringClose(text) => Self::BlockStringClose(text.0),
                    $name::BlockStringSymmetric(delim) => Self::BlockStringSymmetric(delim.0),
                    $name::InlineSpanOpen((text, closing, name)) => Self::InlineSpanOpen { text: text.0, closing: closing.0, name: name.0 },
                    $name::InlineSpanClose(close) => Self::InlineSpanClose(close.0),
                    $name::InlineSpanSymmetric((text, name)) => Self::InlineSpanSymmetric { text: text.0, name: name.0 },
                    $name::BlockSpanOpen((text, closing, name)) => Self::BlockSpanOpen { text: text.0, closing: closing.0, name: name.0 },
                    $name::BlockSpanClose(close) => Self::BlockSpanClose(close.0),
                    $name::BlockSpanSymmetric((text, name)) => Self::BlockSpanSymmetric { text: text.0, name: name.0 },
                    $name::Escape => Self::Escape,
                }
            }
        }

        /// Returns the available tokens (with opening and closing text) for the given filetype.
        pub fn $get_tokens() -> Vec<$crate::languages::AvailableToken> {
            let mut tokens = Vec::new();

            // For delimiter pairs.
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::Delimiter,
                    opening: $open.to_string(),
                    closing: $close.to_string(),
                    name: None,
                });
            )*

            // For block comment pairs.
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::BlockComment,
                    opening: $block_comment_open.to_string(),
                    closing: $block_comment_close.to_string(),
                    name: None,
                });
            )*

            // For block string pairs.
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::String,
                    opening: $block_string_open.to_string(),
                    closing: $block_string_close.to_string(),
                    name: None,
                });
            )*

            // For symmetric block strings (same opening and closing)
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::String,
                    opening: $block_string_symmetric.to_string(),
                    closing: $block_string_symmetric.to_string(),
                    name: None,
                });
            )*

            // For inline span pairs
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::InlineSpan,
                    opening: $inline_span_open.to_string(),
                    closing: $inline_span_close.to_string(),
                    name: Some($inline_span_open_name.to_string()),
                });
            )*

            // For symmetric inline spans
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::InlineSpan,
                    opening: $inline_span_symmetric.to_string(),
                    closing: $inline_span_symmetric.to_string(),
                    name: Some($inline_span_name.to_string()),
                });
            )*

            // For block span pairs
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::BlockSpan,
                    opening: $block_span_open.to_string(),
                    closing: $block_span_close.to_string(),
                    name: Some($block_span_open_name.to_string()),
                });
            )*

            // For symmetric block spans
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::BlockSpan,
                    opening: $block_span_symmetric.to_string(),
                    closing: $block_span_symmetric.to_string(),
                    name: Some($block_span_name.to_string()),
                });
            )*

            tokens
        }
    };

    // Legacy format without inline_span and block_span sections
    ($name:ident, $get_tokens:ident, {
        delimiters: { $($open:literal => $close:literal),* $(,)? },
        line_comment: [ $($line_comment:literal),* $(,)? ],
        block_comment: [ $($block_comment_open:literal => $block_comment_close:literal),* $(,)? ],
        string_regex: [ $($string_regex:literal),* $(,)? ],
        block_string: [
            $(symmetric $block_string_symmetric:literal),*
            $($block_string_open:literal => $block_string_close:literal),* $(,)?
        ]
    }) => {
        // Call the macro with empty inline_span and block_span sections
        $crate::define_token_enum!($name, $get_tokens, {
            delimiters: { $($open => $close),* },
            line_comment: [ $($line_comment),* ],
            block_comment: [ $($block_comment_open => $block_comment_close),* ],
            string_regex: [ $($string_regex),* ],
            block_string: [
                $(symmetric $block_string_symmetric),*
                $($block_string_open => $block_string_close),*
            ],
            inline_span: [],
            block_span: []
        });
    };
}
