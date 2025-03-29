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
mod typescript;
mod typst;
mod zig;

pub use c::CToken;
pub use clojure::ClojureToken;
pub use cpp::CppToken;
pub use csharp::CSharpToken;
pub use dart::DartToken;
pub use elixir::ElixirToken;
pub use erlang::ErlangToken;
pub use fsharp::FSharpToken;
pub use go::GoToken;
pub use haskell::HaskellToken;
pub use haxe::HaxeToken;
pub use java::JavaToken;
pub use javascript::JavaScriptToken;
pub use json::JsonToken;
pub use kotlin::KotlinToken;
pub use latex::LatexToken;
pub use lean::LeanToken;
pub use lua::LuaToken;
pub use objc::ObjCToken;
pub use ocaml::OCamlToken;
pub use perl::PerlToken;
pub use php::PhpToken;
pub use python::PythonToken;
pub use r::RToken;
pub use ruby::RubyToken;
pub use rust::RustToken;
pub use scala::ScalaToken;
pub use shell::ShellToken;
pub use swift::SwiftToken;
pub use toml::TomlToken;
pub use typescript::TypeScriptToken;
pub use typst::TypstToken;
pub use zig::ZigToken;

#[derive(Debug, Clone, Copy)]
pub enum Token {
    DelimiterOpen {
        text: &'static str,
        closing: &'static str,
    },
    DelimiterClose(&'static str),
    LineComment,
    BlockCommentOpen(&'static str),
    BlockCommentClose(&'static str),
    String,
    BlockStringOpen(&'static str),
    BlockStringClose(&'static str),
    BlockStringSymmetric(&'static str),
    Escape,
}

// static string newtype - needed because logos callbacks can't directly return `&'static str`
struct SStr(&'static str);

#[macro_export]
macro_rules! define_token_enum {
    ($name:ident, {
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

            $(#[token($block_comment_open, |_| $crate::languages::SStr($block_comment_open) )])*
            BlockCommentOpen($crate::languages::SStr),
            $(#[token($block_comment_close, |_| $crate::languages::SStr($block_comment_close) )])*
            BlockCommentClose($crate::languages::SStr),

            $(#[regex($string_regex)])*
            String,

            $(#[token($block_string_open, |_| $crate::languages::SStr($block_string_close), priority = 10 )])*
            BlockStringOpen($crate::languages::SStr),
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
                    $name::BlockCommentOpen(closing) => Self::BlockCommentOpen(closing.0),
                    $name::BlockCommentClose(close) => Self::BlockCommentClose(close.0),
                    $name::String => Self::String,
                    $name::BlockStringOpen(closing) => Self::BlockStringOpen(closing.0),
                    $name::BlockStringClose(close) => Self::BlockStringClose(close.0),
                    $name::BlockStringSymmetric(delim) => Self::BlockStringSymmetric(delim.0),
                    $name::Escape => Self::Escape,
                }
            }
        }
    };
}
