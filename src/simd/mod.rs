use std::simd::Simd;

pub type SimdVec = Simd<u8, 32>;

pub mod parse;
pub mod tokenize;

pub use parse::parse;
