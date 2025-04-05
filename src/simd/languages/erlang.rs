use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Erlang {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["%"],
    string: ["\""]
});
