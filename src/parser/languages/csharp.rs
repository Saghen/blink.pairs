use crate::parser::*;
use matcher_macros::define_matcher;

define_matcher!(CSharp {
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
    block_string: ["@\"" => "\""]
});
