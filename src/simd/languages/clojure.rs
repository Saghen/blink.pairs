use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Clojure {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: [";"],
    block_comment: [],
    string: ["\""]
});
