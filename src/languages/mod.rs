mod c;
mod clojure;
mod cpp;
mod csharp;
mod go;
mod java;
mod javascript;
mod json;
mod lua;
mod php;
mod python;
mod ruby;
mod rust;
mod swift;
mod typescript;
mod typst;

pub use c::CToken;
pub use clojure::ClojureToken;
pub use cpp::CppToken;
pub use csharp::CSharpToken;
pub use go::GoToken;
pub use java::JavaToken;
pub use javascript::JavaScriptToken;
pub use json::JsonToken;
pub use lua::LuaToken;
pub use php::PhpToken;
pub use python::PythonToken;
pub use ruby::RubyToken;
pub use rust::RustToken;
pub use swift::SwiftToken;
pub use typescript::TypeScriptToken;
pub use typst::TypstToken;

#[derive(Debug, Clone, Copy)]
pub enum Token<'s> {
    DelimiterOpen(&'s str),
    DelimiterClose(&'s str),
    LineComment,
    BlockCommentOpen,
    BlockCommentClose,
    String(&'s str),
    BlockStringOpen,
    BlockStringClose,
    Escape,
    NewLine,
}

#[macro_export]
macro_rules! define_token_enum {
    ($name:ident, {
        delimiters: { $($open:literal => $close:literal),* $(,)? },
        line_comment: [ $($line_comment:literal),* $(,)? ],
        block_comment: [ $($block_comment_open:literal => $block_comment_close:literal),* $(,)? ],
        string: [ $($string_delim:literal),* $(,)? ],
        block_string: [ $($block_string_open:literal => $block_string_close:literal),* $(,)? ]
    }) => {
        #[allow(unused)] // Ignore warnings about unused variants
        #[derive(logos::Logos)]
        #[logos(skip r"[ \t\f]+")] // Skip whitespace
        pub enum $name<'s> {
            $(#[token($open)])*
            DelimiterOpen(&'s str),

            $(#[token($close)])*
            DelimiterClose(&'s str),

            $(#[token($line_comment)])*
            LineComment,

            $(#[token($block_comment_open)])*
            BlockCommentOpen,
            $(#[token($block_comment_close)])*
            BlockCommentClose,

            $(#[token($string_delim)])*
            String(&'s str),

            $(#[token($block_string_open)])*
            BlockStringOpen,

            $(#[token($block_string_close, priority = 10)])*
            BlockStringClose,

            #[token("\\")]
            Escape,

            #[token("\n")]
            NewLine,
        }

        impl<'s> From<$name<'s>> for $crate::languages::Token<'s> {
            fn from(value: $name<'s>) -> Self {
                match value {
                    $name::DelimiterOpen(s) => Self::DelimiterOpen(s),
                    $name::DelimiterClose(s) => Self::DelimiterClose(s),
                    $name::LineComment => Self::LineComment,
                    $name::BlockCommentOpen => Self::BlockCommentOpen,
                    $name::BlockCommentClose => Self::BlockCommentClose,
                    $name::String(s) => Self::String(s),
                    $name::BlockStringOpen => Self::BlockStringOpen,
                    $name::BlockStringClose => Self::BlockStringClose,
                    $name::Escape => Self::Escape,
                    $name::NewLine => Self::NewLine,
                }
            }
        }
    };
}
