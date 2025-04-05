use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Haskell {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["--"],
    block_comment: ["{-" => "-}"],
    string: ["\""]
});
