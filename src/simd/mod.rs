use std::simd::Simd;

pub type SimdVec = Simd<u8, 64>;

pub mod languages;
pub mod matcher;
pub mod parse;
pub mod tokenize;

pub use matcher::{Match, MatchToken, Matcher};
pub use parse::{parse, State};
pub use tokenize::{tokenize, Token, TokenPos};

pub fn parse_language(
    language: &str,
    lines: &[&str],
    initial_state: State,
) -> Option<(Vec<Vec<Match>>, Vec<State>)> {
    match language {
        "c" => Some(parse(lines, initial_state, languages::C {})),
        _ => None,
    }
}
