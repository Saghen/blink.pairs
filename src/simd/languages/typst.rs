use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Typst {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\"", "'"]
});
