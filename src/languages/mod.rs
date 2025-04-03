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
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum TokenType {
    Delimiter = 0,
    String = 1,
    BlockComment = 2,
}

impl TryFrom<u8> for TokenType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenType::Delimiter),
            1 => Ok(TokenType::String),
            2 => Ok(TokenType::BlockComment),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AvailableToken {
    type_: TokenType,
    opening: String,
    closing: String,
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
                });
            )*

            // For block comment pairs.
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::BlockComment,
                    opening: $block_comment_open.to_string(),
                    closing: $block_comment_close.to_string(),
                });
            )*

            // For block string pairs.
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::String,
                    opening: $block_string_open.to_string(),
                    closing: $block_string_close.to_string(),
                });
            )*

            // For symmetric block strings (same opening and closing)
            $(
                tokens.push($crate::languages::AvailableToken {
                    type_: $crate::languages::TokenType::String,
                    opening: $block_string_symmetric.to_string(),
                    closing: $block_string_symmetric.to_string(),
                });
            )*

            tokens
        }
    };
}
