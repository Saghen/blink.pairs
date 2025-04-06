use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(R {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["#"],
    string: ["\"", "'"]
});
