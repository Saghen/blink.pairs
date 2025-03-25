local watcher = {
  --- @type table<number, boolean>
  watched_bufnrs = {},
}

--- Runs a full parse on the buffer when start_line, old_end_line, and new_end_line are not provided.
--- Otherwise, incrementally parses the buffer.
--- @param bufnr number
--- @param start_line? number
--- @param old_end_line? number
--- @param new_end_line? number
--- @return boolean Whether the buffer is parseable
local function parse_buffer(bufnr, start_line, old_end_line, new_end_line)
  local start_time = vim.uv.hrtime()

  local lines = vim.api.nvim_buf_get_lines(bufnr, start_line or 0, new_end_line or -1, false)

  local rust = require('blink.pairs.rust')
  local did_parse = rust.parse_buffer(bufnr, vim.bo[bufnr].filetype, lines, start_line, old_end_line, new_end_line)

  if did_parse and require('blink.pairs.config').debug then
    vim.print('parsing time: ' .. (vim.uv.hrtime() - start_time) / 1e6 .. ' ms')
  end

  return did_parse
end

--- Runs an initial parse on the buffer and attaches via nvim_buf_attach
--- for incremental parsing
--- @param bufnr number
--- @return boolean is_attached Whether the buffer is parseable and attached
function watcher.attach(bufnr)
  if watcher.watched_bufnrs[bufnr] ~= nil then return true end

  local did_parse = parse_buffer(bufnr)
  if not did_parse then return false end

  watcher.watched_bufnrs[bufnr] = true

  local last_changedtick = 0
  vim.api.nvim_buf_attach(bufnr, false, {
    on_detach = function() watcher.watched_bufnrs[bufnr] = nil end,

    -- Full parse
    on_reload = function() parse_buffer(bufnr) end,
    on_changedtick = function(_, _, changedtick)
      if changedtick == last_changedtick then return end
      last_changedtick = changedtick

      parse_buffer(bufnr)
    end,

    -- Incremental parse
    on_lines = function(_, _, changedtick, start, old_end, new_end)
      if changedtick == last_changedtick then return end
      last_changedtick = changedtick

      local did_incremental_parse = parse_buffer(bufnr, start, old_end, new_end)

      -- no longer parseable, detach
      if not did_incremental_parse then
        watcher.watched_bufnrs[bufnr] = nil
        return true
      end
    end,
  })

  return true
end

return watcher
