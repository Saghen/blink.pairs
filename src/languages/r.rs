use crate::define_token_enum;

define_token_enum!(RToken, r_tokens, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["#"],
    block_comment: [],
    string_regex: ["(?&dstring)", "(?&sstring)"],
    block_string: []
});
