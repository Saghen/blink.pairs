use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Vim {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
});
