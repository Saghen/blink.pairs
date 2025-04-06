use itertools::MultiPeek;
use mlua::IntoLua;

mod token;
mod token_type;

pub use token::*;
pub use token_type::*;

use crate::parser::{State, TokenPos};

pub trait Matcher {
    fn tokens(&self) -> Vec<u8>;

    fn call<I>(
        &mut self,
        matches: &mut Vec<Match>,
        stack: &mut Vec<u8>,
        tokens: &mut MultiPeek<I>,
        state: State,
        token: TokenPos,
    ) -> State
    where
        I: Iterator<Item = TokenPos>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub token: MatchToken,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl Match {
    pub fn new(token: MatchToken, col: usize) -> Self {
        Self {
            token,
            col,
            stack_height: None,
        }
    }

    pub fn with_line(&self, line: usize) -> MatchWithLine {
        MatchWithLine {
            token: self.token.clone(),
            line,
            col: self.col,
            stack_height: self.stack_height,
        }
    }
}

impl From<TokenPos> for Match {
    fn from(token: TokenPos) -> Self {
        Match {
            token: token.byte.into(),
            col: token.col,
            stack_height: None,
        }
    }
}

impl IntoLua for Match {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set("text", self.token.opening())?;
        if let Some(closing) = self.token.closing() {
            table.set("closing", closing)?;
        }

        table.set("col", self.col)?;
        table.set("stack_height", self.stack_height)?;

        (&table).into_lua(lua)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchWithLine {
    pub token: MatchToken,
    pub line: usize,
    pub col: usize,
    pub stack_height: Option<usize>,
}

impl IntoLua for MatchWithLine {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set("text", self.token.opening())?;
        if let Some(closing) = self.token.closing() {
            table.set("closing", closing)?;
        }

        table.set("line", self.line)?;
        table.set("col", self.col)?;
        table.set("stack_height", self.stack_height)?;

        (&table).into_lua(lua)
    }
}
