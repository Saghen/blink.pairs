use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(OCaml {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    block_comment: ["(*" => "*)"],
    string: ["\""]
});
