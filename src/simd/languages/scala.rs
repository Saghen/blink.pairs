use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Scala {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\""],
    block_string: ["\"\"\"" => "\"\"\""]
});
