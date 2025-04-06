use itertools::MultiPeek;
use mlua::IntoLua;

mod token;
mod token_type;

pub use token::*;
pub use token_type::*;

use crate::parser::{CharPos, State};

pub trait Matcher {
    const TOKENS: &[u8];
    #[inline(always)]
    fn tokens(&self) -> &'static [u8] {
        Self::TOKENS
    }

    fn call<I>(
        &mut self,
        matches: &mut Vec<Match>,
        stack: &mut Vec<u8>,
        tokens: &mut MultiPeek<I>,
        state: State,
        token: CharPos,
    ) -> State
    where
        I: Iterator<Item = CharPos>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub kind: Kind,
    pub token: Token,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl Match {
    pub fn new(kind: Kind, token: Token, col: usize) -> Self {
        Self {
            kind,
            token,
            col,
            stack_height: None,
        }
    }

    pub fn new_with_stack(kind: Kind, token: Token, col: usize, stack_height: usize) -> Self {
        Self {
            kind,
            token,
            col,
            stack_height: Some(stack_height),
        }
    }

    pub fn with_line(&self, line: usize) -> MatchWithLine {
        MatchWithLine {
            kind: self.kind,
            token: self.token.clone(),
            line,
            col: self.col,
            stack_height: self.stack_height,
        }
    }

    pub fn delimiter(char: char, col: usize, stack_height: Option<usize>) -> Self {
        let (kind, token) = match char {
            '{' => (Kind::Opening, Token::Delimiter("{", "}")),
            '}' => (Kind::Closing, Token::Delimiter("{", "}")),
            '[' => (Kind::Opening, Token::Delimiter("[", "]")),
            ']' => (Kind::Closing, Token::Delimiter("[", "]")),
            '(' => (Kind::Opening, Token::Delimiter("(", ")")),
            ')' => (Kind::Closing, Token::Delimiter("(", ")")),
            _ => panic!("Unknown token type"),
        };

        Self {
            kind,
            token,
            col,
            stack_height,
        }
    }

    pub fn block_comment(text: &'static str, col: usize) -> Self {
        let (kind, token) = match text {
            "/*" => (Kind::Opening, Token::BlockComment("/*", "*/")),
            "*/" => (Kind::Closing, Token::BlockComment("/*", "*/")),
            _ => panic!("Unknown token type"),
        };
        Self {
            kind,
            token,
            col,
            stack_height: None,
        }
    }

    pub fn line_comment(text: &'static str, col: usize) -> Self {
        Self {
            kind: Kind::NonPair,
            token: Token::LineComment(text),
            col,
            stack_height: None,
        }
    }
}

impl IntoLua for Match {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set(0, self.token.opening())?;
        if let Some(closing) = self.token.closing() {
            table.set(1, closing)?;
        }
        table.set("col", self.col)?;
        table.set("stack_height", self.stack_height)?;

        (&table).into_lua(lua)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchWithLine {
    pub kind: Kind,
    pub token: Token,
    pub line: usize,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl IntoLua for MatchWithLine {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set(0, self.token.opening())?;
        if let Some(closing) = self.token.closing() {
            table.set(1, closing)?;
        }
        table.set("line", self.line)?;
        table.set("col", self.col)?;
        table.set("stack_height", self.stack_height)?;

        (&table).into_lua(lua)
    }
}
