//! `<` and `>` are ambiguous with `a < b` comparison operators, so they can't be matched like
//! regular delimiters. Instead, we rely on the surrounding whitespace to disambiguate. This relies
//! on the code being formatted but avoids language-specific parsing.
//!
//! Vec<T>  -> no space, delimiter
//! a < b   -> space surrounding, not a delimiter

/// Whether the `<` at `col` opens a pair, rather than being an operator like `<`, `<=` or `<<`
///
/// ```text
/// Vec<T>                          -> opening
/// <T as Trait>::foo()             -> opening
/// <<T as Trait>::Item as Foo>::X  -> both opening
/// fn foo<                         -> opening (multi-line generics)
/// a < b, a <= b, a << b, 0..<n    -> not opening
/// ```
pub fn is_angle_bracket_opening(line: &[u8], col: usize) -> bool {
    // Swift ranges like `0..<n`, but not JSX text like `Loading...</div>`
    if line[..col].ends_with(b"..") && line.get(col + 1) != Some(&b'/') {
        return false;
    }

    match line.get(col + 1) {
        // `a < b`, `a <= b`
        Some(b' ' | b'\t' | b'=') => false,
        // `a << b`, `a <<= b`, unless it's a qualified path like `<<T as Trait>::Item as Foo>::X`
        Some(b'<') => !matches!(line.get(col + 2), Some(b' ' | b'\t' | b'=') | None),
        // Includes the end of the line, for generics spanning multiple lines
        _ => true,
    }
}

/// Whether the `>` at `col` closes a pair, rather than being an operator like `>`, `>=`, `>>`,
/// `->` or `=>`
///
/// ```text
/// Vec<T>                          -> closing
/// Vec<Vec<T>>                     -> both closing
/// >() {                           -> closing (multi-line generics)
/// a > b, a >= b, a >> b           -> not closing
/// fn() -> T, x => y               -> not closing
/// ```
pub fn is_angle_bracket_closing(line: &[u8], col: usize) -> bool {
    // `a >= b`
    if line.get(col + 1) == Some(&b'=') {
        return false;
    }

    match col.checked_sub(1).map(|prev| line[prev]) {
        // `a > b`, unless it's the first character on the line, for generics spanning multiple lines
        Some(b' ' | b'\t') => line[..col].iter().all(|b| matches!(b, b' ' | b'\t')),
        // `->`, `=>`
        Some(b'-' | b'=') => false,
        // `a >> b`, unless the previous `>` also closed a pair like `Vec<Vec<T>>`
        Some(b'>') => is_angle_bracket_closing(line, col - 1),
        _ => true,
    }
}
