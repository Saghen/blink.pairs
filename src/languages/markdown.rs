use crate::define_token_enum;

define_token_enum!(MarkdownToken, markdown_tokens, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: [],
    block_comment: [],
    string_regex: ["(?&dstring)", "(?&sstring)"],
    block_string: [],
    inline_span: [
        symmetric "$" => "math",
        symmetric "_" => "italic", 
        symmetric "*" => "italic",
        symmetric "**" => "bold",
        symmetric "~~" => "strikethrough"
    ],
    block_span: [
        symmetric "$$" => "math",
        symmetric "```" => "code"
    ]
}); 