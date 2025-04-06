use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Toml {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    line_comment: ["#"],
    string: ["\""],
    block_string: ["\"\"\"" => "\"\"\""]
});
