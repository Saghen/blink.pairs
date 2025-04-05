use crate::simd::*;
use matcher_macros::define_matcher;

define_matcher!(Zig {
    delimiters: [
        "(" => ")",
        "[" => "]",
        "{" => "}"
    ],
    // HACK: In Zig, multiline string literals are consecutive lines that start with the \\ token.
    // They do not have distinct open and close delimiters, so they treated as line comments.
    line_comment: ["//", "\\\\"],
    string: ["\""]
});
