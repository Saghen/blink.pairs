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

pub enum Token {
    DelimiterOpen,
    DelimiterClose,
    LineComment,
    BlockCommentOpen,
    BlockCommentClose,
    String,
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
        pub enum $name {
            $(#[token($open)])*
            DelimiterOpen,

            $(#[token($close)])*
            DelimiterClose,

            $(#[token($line_comment)])*
            LineComment,

            $(#[token($block_comment_open)])*
            BlockCommentOpen,
            $(#[token($block_comment_close)])*
            BlockCommentClose,

            $(#[token($string_delim)])*
            String,

            $(#[token($block_string_open)])*
            BlockStringOpen,

            $(#[token($block_string_close, priority = 10)])*
            BlockStringClose,

            #[token("\\")]
            Escape,

            #[token("\n")]
            NewLine,
        }

        impl From<$name> for $crate::languages::Token {
            fn from(value: $name) -> Self {
                match value {
                    $name::DelimiterOpen => Self::DelimiterOpen,
                    $name::DelimiterClose => Self::DelimiterClose,
                    $name::LineComment => Self::LineComment,
                    $name::BlockCommentOpen => Self::BlockCommentOpen,
                    $name::BlockCommentClose => Self::BlockCommentClose,
                    $name::String => Self::String,
                    $name::BlockStringOpen => Self::BlockStringOpen,
                    $name::BlockStringClose => Self::BlockStringClose,
                    $name::Escape => Self::Escape,
                    $name::NewLine => Self::NewLine,
                }
            }
        }
    };
}
