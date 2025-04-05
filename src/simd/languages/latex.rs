use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Latex {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["%"],
    string: ["\""],
    char: ["'"],
    block_string: ["$" => "$", "$$" => "$$"]
});
