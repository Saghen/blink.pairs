use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Ruby {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["#"],
    block_comment: ["=begin" => "end"],
    string: ["\"", "'"]
});
