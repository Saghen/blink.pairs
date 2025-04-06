use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Rust {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    char: ["'"],
    block_string: [
        "\"" => "\"",
        "r#\"" => "\"#",
        "r##\"" => "\"##",
        "r###\"" => "\"###"
    ]
});
