use buffer::ParsedBuffer;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};

use parser::Match;

pub mod buffer;
pub mod languages;
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
    (bufnr, filetype, text, start_line, old_end_line, new_end_line): (
        usize,
        String,
        String,
        Option<usize>,
        Option<usize>,
        Option<usize>,
    ),
) -> LuaResult<bool> {
    let mut parsed_buffers = get_parsed_buffers();

    // Incremental parse
    if let Some(parsed_buffer) = parsed_buffers.get_mut(&bufnr) {
        Ok(parsed_buffer.reparse_range(&filetype, &text, start_line, old_end_line, new_end_line))
    }
    // Full parse
    else if let Some(parsed_buffer) = ParsedBuffer::parse(&filetype, &text) {
        parsed_buffers.insert(bufnr, parsed_buffer);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_parsed_line(_lua: &Lua, (bufnr, line_number): (usize, usize)) -> LuaResult<Vec<Match>> {
    let parsed_buffers = get_parsed_buffers();

    if let Some(parsed_buffer) = parsed_buffers.get(&bufnr) {
        if let Some(line_matches) = parsed_buffer.line_matches(line_number) {
            return Ok(line_matches);
        }
    }

    Ok(Vec::new())
}

// NOTE: skip_memory_check greatly improves performance
// https://github.com/mlua-rs/mlua/issues/318
#[mlua::lua_module(skip_memory_check)]
fn blink_pairs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("parse_buffer", lua.create_function(parse_buffer)?)?;
    exports.set("get_parsed_line", lua.create_function(get_parsed_line)?)?;
    Ok(exports)
}
