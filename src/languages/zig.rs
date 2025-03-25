use crate::define_token_enum;

define_token_enum!(ZigToken, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    // HACK: In Zig, multiline string literals are consecutive lines that start with the \\ token.
    // They do not have distinct open and close delimiters, so they treated as line comments.
    line_comment: ["//", "\\\\"],
    block_comment: [],
    string: ["\""],
    block_string: []
});
