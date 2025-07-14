//! Module for calculating indentation levels in source code.

use std::simd::{cmp::SimdPartialEq, Simd};

/// Calculate indentation levels with a custom tab width.
///
/// Returns a vector where each element represents the indentation level
/// (in spaces) of a line that contains non-whitespace characters.
/// Lines that are empty or contain only whitespace are not included.
///
/// # Examples
///
/// ```
/// use blink_pairs::indent::calculate_indentation_with_tab_width;
///
/// let src = "fn main() {\n\tprintln!(\"hello\");\n}";
/// let indents = calculate_indentation_with_tab_width(src, 8);
/// assert_eq!(indents, vec![0, 8, 0]);
/// ```
pub fn calculate_indentation_with_tab_width(src: &str, tab_width: u8) -> Vec<u8> {
    let mut in_indentation = true;
    let mut indentation = vec![];
    let mut current_indentation = 0;
    let bytes = src.as_bytes();
    let (chunks, rem) = bytes.as_chunks::<32>();

    for &chunk in chunks {
        let chunk = Simd::from_array(chunk);
        let newlines: Simd<u8, _> = chunk
            .simd_eq(Simd::splat(b'\n'))
            .select(Simd::splat(0xFF), Simd::splat(0));
        let spaces = chunk
            .simd_eq(Simd::splat(b' '))
            .select(Simd::splat(1), Simd::splat(0));
        let tabs = chunk
            .simd_eq(Simd::splat(b'\t'))
            .select(Simd::splat(tab_width), Simd::splat(0));
        let tokens = newlines + spaces + tabs;

        for c in tokens.to_array() {
            if c == 0xFF {
                in_indentation = true;
            } else if in_indentation {
                if c > 0 {
                    current_indentation += c;
                } else {
                    in_indentation = false;
                    indentation.push(current_indentation);
                    current_indentation = 0;
                }
            }
        }
    }

    for &c in rem {
        if c == b'\n' {
            in_indentation = true;
        } else if in_indentation {
            match c {
                b' ' => current_indentation += 1,
                b'\t' => current_indentation += tab_width,
                _ => {
                    in_indentation = false;
                    indentation.push(current_indentation);
                    current_indentation = 0;
                }
            }
        }
    }

    if in_indentation {
        indentation.push(current_indentation);
    }

    indentation
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn calculate_indentation(src: &str) -> Vec<u8> {
        calculate_indentation_with_tab_width(src, TAB_WIDTH)
    }

    #[test]
    fn test_basic_indentation() {
        let src = "if foo() {\n    bar();\n}";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4, 0]);
    }

    #[test]
    fn test_mixed_tabs_and_spaces() {
        let src = "if foo() {\n    if bar {\n\t\tprintln!(\"hello world\");\n    }\n}";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4, 8, 4, 0]);
    }

    #[test]
    fn test_empty_lines() {
        let src = "line1\n\n    line3\n\n        line5";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4, 8]);
    }

    #[test]
    fn test_all_whitespace_lines() {
        let src = "line1\n    \n\t\n    line4";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4]);
    }

    #[test]
    fn test_custom_tab_width() {
        let src = "\tindented\n\t\tdouble";
        let result = calculate_indentation_with_tab_width(src, 8);
        assert_eq!(result, vec![8, 16]);
    }

    #[test]
    fn test_no_trailing_newline() {
        let src = "line1\n    line2";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4]);
    }

    #[test]
    fn test_only_whitespace() {
        let src = "    ";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![4]);
    }

    #[test]
    fn test_empty_string() {
        let src = "";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_single_line_no_indentation() {
        let src = "hello world";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_large_input() {
        // Test with input larger than SIMD chunk size (32 bytes)
        let src = "a".repeat(40) + "\n" + &" ".repeat(40) + "b";
        let result = calculate_indentation(&src);
        assert_eq!(result, vec![0, 40]);
    }

    #[test]
    fn test_windows_line_endings() {
        // The current implementation treats \r as a non-whitespace character
        let src = "line1\r\n    line2\r\n";
        let result = calculate_indentation(src);
        assert_eq!(result, vec![0, 4]);
    }
}
