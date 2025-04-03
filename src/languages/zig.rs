use crate::define_token_enum;

define_token_enum!(ZigToken, zig_tokens, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    // HACK: In Zig, multiline string literals are consecutive lines that start with the \\ token.
    // They do not have distinct open and close delimiters, so they treated as line comments.
    line_comment: ["//", "\\\\"],
    block_comment: [],
    string_regex: ["(?&dstring)"],
    block_string: []
});
