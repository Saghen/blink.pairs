pub type SimdVec = std::simd::Simd<u8, 16>;

pub mod languages;
pub mod matcher;
pub mod parse;
pub mod tokenize;

pub use itertools::MultiPeek;
pub use matcher::{Kind, Match, MatchWithLine, Matcher, Token};
pub use parse::{parse, State};
pub use tokenize::{tokenize, CharPos};

pub fn parse_filetype(
    filetype: &str,
    lines: &[&str],
    initial_state: State,
) -> Option<(Vec<Vec<Match>>, Vec<State>)> {
    match filetype {
        "c" => Some(parse(lines, initial_state, languages::C {})),
        "clojure" => Some(parse(lines, initial_state, languages::Clojure {})),
        "cpp" => Some(parse(lines, initial_state, languages::Cpp {})),
        "csharp" => Some(parse(lines, initial_state, languages::CSharp {})),
        "dart" => Some(parse(lines, initial_state, languages::Dart {})),
        "elixir" => Some(parse(lines, initial_state, languages::Elixir {})),
        "erlang" => Some(parse(lines, initial_state, languages::Erlang {})),
        "fsharp" => Some(parse(lines, initial_state, languages::FSharp {})),
        "go" => Some(parse(lines, initial_state, languages::Go {})),
        "haskell" => Some(parse(lines, initial_state, languages::Haskell {})),
        "haxe" => Some(parse(lines, initial_state, languages::Haxe {})),
        "java" => Some(parse(lines, initial_state, languages::Java {})),
        "javascript" => Some(parse(lines, initial_state, languages::JavaScript {})),
        "json" => Some(parse(lines, initial_state, languages::Json {})),
        "kotlin" => Some(parse(lines, initial_state, languages::Kotlin {})),
        "latex" => Some(parse(lines, initial_state, languages::Latex {})),
        "lean" => Some(parse(lines, initial_state, languages::Lean {})),
        "lua" => Some(parse(lines, initial_state, languages::Lua {})),
        "markdown" => Some(parse(lines, initial_state, languages::Markdown {})),
        "objc" => Some(parse(lines, initial_state, languages::ObjC {})),
        "ocaml" => Some(parse(lines, initial_state, languages::OCaml {})),
        "perl" => Some(parse(lines, initial_state, languages::Perl {})),
        "php" => Some(parse(lines, initial_state, languages::Php {})),
        "python" => Some(parse(lines, initial_state, languages::Python {})),
        "r" => Some(parse(lines, initial_state, languages::R {})),
        "ruby" => Some(parse(lines, initial_state, languages::Ruby {})),
        "rust" => Some(parse(lines, initial_state, languages::Rust {})),
        "scala" => Some(parse(lines, initial_state, languages::Scala {})),
        "shell" => Some(parse(lines, initial_state, languages::Shell {})),
        "swift" => Some(parse(lines, initial_state, languages::Swift {})),
        "toml" => Some(parse(lines, initial_state, languages::Toml {})),
        "typst" => Some(parse(lines, initial_state, languages::Typst {})),
        "zig" => Some(parse(lines, initial_state, languages::Zig {})),

        _ => None,
    }
}
