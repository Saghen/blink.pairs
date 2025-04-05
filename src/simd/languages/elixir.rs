use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Elixir {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["#"],
    string: ["\""],
    block_string: ["\"\"\"" => "\"\"\""]
});
