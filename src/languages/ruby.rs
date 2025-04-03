use crate::define_token_enum;

define_token_enum!(RubyToken, ruby_tokens, {
    delimiters: {
        "(" => ")",
        "[" => "]",
        "{" => "}"
    },
    line_comment: ["#"],
    block_comment: ["=begin" => "end"],
    string_regex: ["(?&dstring)", "(?&sstring)"],
    block_string: []
});
