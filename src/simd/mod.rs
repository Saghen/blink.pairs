use std::simd::Simd;

pub type SimdVec = Simd<u8, 64>;

pub mod languages;
pub mod parse;
pub mod tokenize;

pub use parse::{parse, State};
