use crate::define_token_enum;

define_token_enum!(RToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["#"],
    block_comment: [],
    string: ["\"", "'"],
    block_string: []
});
