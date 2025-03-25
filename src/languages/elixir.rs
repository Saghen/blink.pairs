use crate::define_token_enum;

define_token_enum!(ElixirToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["#"],
    block_comment: [],
    string: ["\""],
    block_string: ["\"\"\"" => "\"\"\""]
});
