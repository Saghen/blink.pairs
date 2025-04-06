use crate::parser::*;
use matcher_macros::define_matcher;

// Includes comments from jsonc and json5
define_matcher!(Json {
    delimiters: [
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\""]
});
