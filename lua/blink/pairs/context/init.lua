--- @class blink.pairs.Context
--- @field mode string
--- @field ft string filetype
--- @field bufnr integer buffer id
--- @field cursor { row: integer, col: integer } cursor position
--- @field line string current line contents
--- @field parser blink.pairs.Parser
--- @field char_under_cursor string character under the cursor
--- @field prev_non_ws_col integer column of the last non-whitespace character before the cursor
--- @field ts blink.pairs.context.Treesitter
local Context = {}

--- @private
--- @type table<string, fun(ctx: blink.pairs.Context): ...>
Context.__field_constructors = {
  char_under_cursor = function(ctx) return ctx.line:sub(ctx.cursor.col, ctx.cursor.col) end,
  prev_non_ws_col = function(ctx)
    for i = ctx.cursor.col, 1, -1 do
      if not ctx.line:sub(i, i):match('%s') then return i end
    end
    return 0
  end,
}

--- @private
Context.__mt = {
  __index = function(ctx, key)
    if Context[key] ~= nil then
      return Context[key]
    elseif Context.__field_constructors[key] ~= nil then
      local value = Context.__field_constructors[key](ctx)
      rawset(ctx, key, value)
      return value
    end
  end,
}

--- Extract a substring around the cursor.
--- @param self blink.pairs.Context
--- @param start_offset integer
--- @param end_offset integer
--- @return string
function Context:text_around_cursor(start_offset, end_offset)
  return self.line:sub(self.cursor.col + start_offset + 1, self.cursor.col + end_offset)
end

--- Extract up to `chars` characters immediately before the cursor. If `chars`
--- is nil, returns text from the beginning of the line up to the cursor.
--- @param self blink.pairs.Context
--- @param chars integer?
--- @return string
function Context:text_before_cursor(chars)
  return self.line:sub(chars and (self.cursor.col - chars + 1) or 1, self.cursor.col)
end

--- Extract up to `chars` characters immediately after the cursor.
--- If `chars` is nil, returns text from just after the cursor to the end of the line.
--- @param self blink.pairs.Context
--- @param chars integer?
--- @return string
function Context:text_after_cursor(chars)
  return self.line:sub(self.cursor.col + 1, chars and (self.cursor.col + chars) or nil)
end

--- Checks if the text after the cursor is equal to the given text.
--- @param self blink.pairs.Context
--- @param text string
--- @param ignore_single_space? boolean
--- @return boolean
function Context:is_after_cursor(text, ignore_single_space)
  assert(text ~= '', 'Text must not be empty')

  local col = self.cursor.col
  if ignore_single_space then
    if self.line:sub(col + 1, col + 1) == ' ' then col = col + 1 end
  end

  return self.line:sub(col + 1, col + #text) == text
end

--- Checks if the text before the cursor is equal to the given text.
--- @param self blink.pairs.Context
--- @param text string
--- @param ignore_single_space? boolean
--- @return boolean
function Context:is_before_cursor(text, ignore_single_space)
  assert(text ~= '', 'Text must not be empty')

  local col = self.cursor.col
  if ignore_single_space then
    if self.char_under_cursor == ' ' then col = col - 1 end
  end

  return self.line:sub(col - #text + 1, col) == text
end

---@return string
local get_mode = function() return vim.api.nvim_get_mode().mode end

---@param mode string?
---@return integer
local get_bufnr = function(mode)
  mode = mode or get_mode()
  if mode == 'c' and pcall(require, 'vim._extui') then
    local bufnr = require('vim._extui.shared').bufs.cmd
    if bufnr and bufnr ~= -1 then return bufnr end
  end
  return vim.api.nvim_get_current_buf()
end

---@param mode string?
---@return [integer, integer]
local get_cursor = function(mode)
  mode = mode or get_mode()
  if mode == 'c' then
    return { 1, vim.fn.getcmdpos() - 1 }
  else
    return vim.api.nvim_win_get_cursor(0)
  end
end

---@param mode string?
---@return string
local get_line = function(mode)
  mode = mode or get_mode()
  return mode == 'c' and vim.fn.getcmdline() or vim.api.nvim_get_current_line()
end

local M = {}

--- @return blink.pairs.Context
function M.new()
  local mode = get_mode()
  local cursor = get_cursor(mode)
  local bufnr = get_bufnr(mode)
  local self = {
    mode = mode,
    ft = vim.bo[bufnr].filetype,
    bufnr = bufnr,
    cursor = { row = cursor[1], col = cursor[2] },
    line = get_line(mode),
    parser = require('blink.pairs.rust'),
  }
  ---@diagnostic disable-next-line: invisible
  self.ts = setmetatable({ ctx = self }, require('blink.pairs.context.treesitter').__mt)
  return setmetatable(self, Context.__mt)
end

return M
