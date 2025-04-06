use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Swift {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\"", "'"],
    block_string: ["\"\"\"" => "\"\"\""]
});
