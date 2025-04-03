use crate::define_token_enum;

// Includes comments from jsonc and json5
define_token_enum!(JsonToken, json_tokens, {
    delimiters: {
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["//"],
    block_comment: ["/*" => "*/"],
    string_regex: ["(?&dstring)"],
    block_string: []
});
