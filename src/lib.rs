use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};

use parser::{parse_filetype, recalculate_stack_heights, Match};

pub mod languages;
pub mod parser;

static PARSED_BUFFERS: LazyLock<Mutex<HashMap<i32, Vec<Vec<Match>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_parsed_buffers<'a>() -> MutexGuard<'a, HashMap<i32, Vec<Vec<Match>>>> {
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
        i32,
        String,
        String,
        Option<i32>,
        Option<i32>,
        Option<i32>,
    ),
) -> LuaResult<bool> {
    let mut parsed_buffers = get_parsed_buffers();

    // Incremental parse
    if let Some(existing_matches_by_line) = parsed_buffers.get_mut(&bufnr) {
        let max_line = existing_matches_by_line.len() as i32;
        let start_line = start_line.unwrap_or(0).min(max_line) as usize;
        let old_end_line = old_end_line.unwrap_or(max_line).min(max_line) as usize;
        let old_range = start_line..old_end_line;

        return match parse_filetype(filetype, text) {
            None => Ok(false),
            Some(matches_by_line) => {
                let new_end_line =
                    new_end_line.unwrap_or((start_line + matches_by_line.len()) as i32) as usize;
                let length = new_end_line - start_line;

                existing_matches_by_line.splice(old_range, matches_by_line[0..length].to_vec());
                recalculate_stack_heights(existing_matches_by_line);
                Ok(true)
            }
        };
    }
    // Full parse
    else if let Some(matches_by_line) = parse_filetype(filetype, text) {
        parsed_buffers.insert(bufnr, matches_by_line);
        return Ok(true);
    }

    Ok(false)
}

fn get_parsed_line(_lua: &Lua, (bufnr, line_number): (i32, i32)) -> LuaResult<Vec<Match>> {
    let parsed_buffers = get_parsed_buffers();

    if let Some(parsed_buffer) = parsed_buffers.get(&bufnr) {
        if let Some(line_matches) = parsed_buffer.get(line_number as usize) {
            return Ok(line_matches.clone());
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
