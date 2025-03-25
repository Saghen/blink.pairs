use crate::define_token_enum;

define_token_enum!(TypeScriptToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\"", "'"],
    block_string: [symmetric "`"]
});
