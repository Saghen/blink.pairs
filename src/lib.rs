#![feature(portable_simd)]

use mlua::prelude::*;
use parser::matcher::TokenType;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};

use buffer::ParsedBuffer;
use parser::{Match, MatchWithLine};

pub mod buffer;
pub mod parser;

static PARSED_BUFFERS: LazyLock<Mutex<HashMap<usize, ParsedBuffer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_parsed_buffers<'a>() -> MutexGuard<'a, HashMap<usize, ParsedBuffer>> {
    match PARSED_BUFFERS.lock() {
        Ok(lock) => lock,
        Err(_) => {
            // Reset the mutex
            PARSED_BUFFERS.clear_poison();
            let mut parsed_buffers = PARSED_BUFFERS.lock().unwrap();
            *parsed_buffers = HashMap::new();
            parsed_buffers
        }
    }
}

fn parse_buffer(
    _lua: &Lua,
    (bufnr, filetype, lines, start_line, old_end_line, new_end_line): (
        usize,
        String,
        Vec<String>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
    ),
) -> LuaResult<bool> {
    let lines_ref = lines.iter().map(|str| str.as_ref()).collect::<Vec<_>>();

    let mut parsed_buffers = get_parsed_buffers();

    // Incremental parse
    if let Some(parsed_buffer) = parsed_buffers.get_mut(&bufnr) {
        Ok(parsed_buffer.reparse_range(
            &filetype,
            &lines_ref,
            start_line,
            old_end_line,
            new_end_line,
        ))
    }
    // Full parse
    else if let Some(parsed_buffer) = ParsedBuffer::parse(&filetype, &lines_ref) {
        parsed_buffers.insert(bufnr, parsed_buffer);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_line_matches(
    _lua: &Lua,
    (bufnr, line_number, token_type): (usize, usize, Option<u8>),
) -> LuaResult<Vec<Match>> {
    let parsed_buffers = get_parsed_buffers();
    let token_type = token_type
        // TODO: don't ignore the error
        .and_then(|token_type| token_type.try_into().ok())
        .unwrap_or(TokenType::Delimiter);

    if let Some(parsed_buffer) = parsed_buffers.get(&bufnr) {
        if let Some(line_matches) = parsed_buffer.line_matches(line_number) {
            return Ok(line_matches
                .iter()
                .filter(|m| token_type.matches(&m.token))
                .cloned()
                .collect());
        }
    }

    Ok(Vec::new())
}

fn get_span_at(_lua: &Lua, (bufnr, row, col): (usize, usize, usize)) -> LuaResult<Option<String>> {
    Ok(get_parsed_buffers()
        .get(&bufnr)
        .and_then(|parsed_buffer| parsed_buffer.span_at(row, col)))
}

fn get_match_at(_lua: &Lua, (bufnr, row, col): (usize, usize, usize)) -> LuaResult<Option<Match>> {
    Ok(get_parsed_buffers()
        .get(&bufnr)
        .and_then(|parsed_buffer| parsed_buffer.match_at(row, col)))
}

fn get_match_pair(
    _lua: &Lua,
    (bufnr, row, col): (usize, usize, usize),
) -> LuaResult<Option<Vec<MatchWithLine>>> {
    Ok(get_parsed_buffers()
        .get(&bufnr)
        .and_then(|parsed_buffer| parsed_buffer.match_pair(row, col))
        .map(|(open, close)| vec![open, close]))
}

// NOTE: skip_memory_check greatly improves performance
// https://github.com/mlua-rs/mlua/issues/318
#[mlua::lua_module(skip_memory_check)]
fn blink_pairs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("parse_buffer", lua.create_function(parse_buffer)?)?;
    exports.set("get_line_matches", lua.create_function(get_line_matches)?)?;
    exports.set("get_span_at", lua.create_function(get_span_at)?)?;
    exports.set("get_match_at", lua.create_function(get_match_at)?)?;
    exports.set("get_match_pair", lua.create_function(get_match_pair)?)?;
    Ok(exports)
}
