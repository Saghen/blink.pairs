use crate::define_token_enum;

define_token_enum!(RustToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string: ["\""],
    block_string: ["r#\"" => "\"#", "r##\"" => "##\"", "r###\"" => "###\""]
});
