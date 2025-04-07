use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates the match header for the given lookahead
///
/// Examples:
///
/// lookahead = 2
/// Generates: (state, token.byte, token_1_byte, token_2_byte)
///
/// lookahead = 0
/// Generates: (state, token.byte)
pub fn create_match_header(lookahead: usize) -> TokenStream2 {
    let mut pattern_str = "(state, token.byte".to_string();

    for i in 0..lookahead {
        pattern_str.push_str(&format!(", token_{}_byte", i + 1));
    }

    pattern_str.push_str(", escaped");

    pattern_str.push(')');

    pattern_str.parse().unwrap()
}

/// Create a match arm for matching against a string
/// with the appropriate number of wildcards
///
/// Examples:
///
/// max_lookahead = 2
/// pattern = "{"
///
/// Generates: (#state, b'{', _, _) if #cond => { #body }
///
/// max_lookahead = 3
/// pattern = "//"
///
/// Generates: (#state, b'/', b'/', _, _) if #cond => { #body }
pub struct MatchArm {
    pattern: String,
    lookahead: usize,
    adjacent: bool,
    _input_state: TokenStream2,
    _ignore_escaped: bool,
    _if_condition: Option<TokenStream2>,
    _body: Option<TokenStream2>,
}

impl MatchArm {
    pub fn builder(pattern: String, lookahead: usize) -> Self {
        let adjacent = pattern.len() > 1;
        Self {
            pattern,
            lookahead,
            adjacent,
            _input_state: quote! { State::Normal },
            _ignore_escaped: false,
            _if_condition: None,
            _body: None,
        }
    }

    pub fn input_state(mut self, input_state: TokenStream2) -> Self {
        self._input_state = input_state;
        self
    }

    pub fn non_adjacent(mut self) -> Self {
        self.adjacent = false;
        self
    }

    pub fn ignore_escaped(mut self) -> Self {
        self._ignore_escaped = true;
        self
    }

    pub fn if_condition(mut self, if_condition: TokenStream2) -> Self {
        self._if_condition = Some(if_condition);
        self
    }

    pub fn body(mut self, body: TokenStream2) -> Self {
        self._body = Some(body);
        self
    }

    pub fn adjacent_if_condition(pattern: &str) -> TokenStream2 {
        let mut if_pattern = "token_1_distance == 1".to_string();

        for i in 2..pattern.len() {
            if_pattern.push_str(&format!(" && token_{}_distance == {} ", i, i));
        }

        if_pattern.parse().unwrap()
    }

    pub fn build(self) -> TokenStream2 {
        // Start with state and main byte
        let mut condition = format!("({}", self._input_state);

        // Add pattern bytes
        for byte in self.pattern.as_bytes().iter() {
            condition.push_str(&format!(", {}", byte));
        }

        // Add `_` for each lookahead token we didn't use
        for _ in 0..(self.lookahead - (self.pattern.len() - 1)) {
            condition.push_str(", _");
        }

        // Add escaped condition
        if self._ignore_escaped {
            condition.push_str(", false");
        } else {
            condition.push_str(", _");
        }

        condition.push(')');
        let mut condition: TokenStream2 = condition.parse().unwrap();

        // Add if statement
        if self.adjacent || self._if_condition.is_some() {
            condition.extend(quote! { if });
            if self.adjacent {
                condition.extend(Self::adjacent_if_condition(&self.pattern));
            }
            if let Some(if_condition) = self._if_condition {
                condition.extend(if_condition);
            }
        }

        // Combine condition and body
        let body = self._body.unwrap();
        quote! { #condition => { #body } }
    }
}
