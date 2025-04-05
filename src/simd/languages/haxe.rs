use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Haxe {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    char: ["'"],
    string: ["\""]
});
