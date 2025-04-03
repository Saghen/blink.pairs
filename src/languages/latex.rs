use crate::define_token_enum;

define_token_enum!(LatexToken, latex_tokens, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["%"],
    block_comment: [],
    string_regex: ["(?&dstring)", "(?&schar)"],
    block_string: [
        symmetric "\\$",
        symmetric "\\$\\$"
    ]
});
