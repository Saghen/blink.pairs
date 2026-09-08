use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(Java {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    angle_brackets: ["<" => ">"],
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    char: ["'"],
    string: ["\""],
    block_string: ["\"\"\"" => "\"\"\""]
});
