use std::simd::Simd;

pub type SimdVec = Simd<u8, 16>;

pub mod parse;
pub mod tokenize;

pub use parse::parse;
