use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Shell {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["#"],
    string: ["\"", "'"]
});
