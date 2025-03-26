use crate::define_token_enum;

define_token_enum!(LeanToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["--"],
    block_comment: ["/-" => "-/"],
    string_regex: ["(?&dstring)"],
    block_string: []
});
