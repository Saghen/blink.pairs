use crate::define_token_enum;

define_token_enum!(CppToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\"", "'"],
    block_string: ["R\"(" => ")\""]
});
