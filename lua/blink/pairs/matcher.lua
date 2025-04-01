-- matcher.lua - Implements different matching strategies
local M = {}

local function rfind(str, pattern, plain)
  for i = #str, 1, -1 do
    local match_start, match_end = str:find(pattern, i, plain)
    if match_start and match_start <= i then return match_start, match_end end
  end
  return nil
end

--- Try TreeSitter-based matching
--- @param ctx table Context with bufnr, row, col, config, and delimiters
--- @return boolean Whether a match was found
function M.try_treesitter_match(ctx)
  local ok, parser = pcall(vim.treesitter.get_parser, ctx.bufnr)
  if not ok or not parser then return false end

  local bufnr = ctx.bufnr
  local row = ctx.row
  local col = ctx.col
  local config = ctx.config
  local delimiters = ctx.delimiters
  local create_highlight = ctx.create_highlight

  local root = parser:parse()[1]:root()
  local node = root:named_descendant_for_range(row, col, row, col + 1)
  if not node then return false end

  local node_text = vim.treesitter.get_node_text(node, bufnr)
  local node_type = node:type()

  local delimiter_utils = require('blink.pairs.delimiters')
  local is_delimiter = delimiter_utils.is_delimiter(node_text, delimiters)
    or delimiter_utils.is_delimiter(node_type, delimiters)
  if not is_delimiter then return false end

  local parent = node:parent()
  if not parent then return false end

  local matching_node = nil
  local matching_text = delimiter_utils.get_matching(node_text, delimiters)
  local matching_type = delimiter_utils.get_matching(node_type, delimiters)

  for child in parent:iter_children() do
    if child ~= node then
      local child_type = child:type()
      local child_text = vim.treesitter.get_node_text(child, bufnr)

      if (matching_text and matching_text == child_text) or (matching_type and matching_type == child_type) then
        matching_node = child
        break
      end
    end
  end

  if not matching_node then return false end

  local marks = {}
  local start_row, start_col, end_row, end_col = node:range()
  table.insert(marks, create_highlight(bufnr, start_row, start_col, end_col, config))

  local m_start_row, m_start_col, m_end_row, m_end_col = matching_node:range()
  table.insert(marks, create_highlight(bufnr, m_start_row, m_start_col, m_end_col, config))

  ctx.store_marks(bufnr, marks)
  return true
end

--- Find matching treesitter node for a delimiter
--- @param node table Current treesitter node
--- @param parent table Parent node
--- @param delimiters table Delimiter information
--- @param bufnr number Buffer number
--- @return table|nil Matching node or nil if none found
function M.find_matching_ts_node(node, parent, delimiters, bufnr)
  local node_text = vim.treesitter.get_node_text(node, bufnr)
  local node_type = node:type()

  local delimiter_utils = require('blink.pairs.delimiters')
  local matching_text = delimiter_utils.get_matching(node_text, delimiters)
  local matching_type = delimiter_utils.get_matching(node_type, delimiters)

  for child in parent:iter_children() do
    if child ~= node then
      local child_type = child:type()
      local child_text = vim.treesitter.get_node_text(child, bufnr)

      if (matching_text and matching_text == child_text) or (matching_type and matching_type == child_type) then
        return child
      end
    end
  end

  return nil
end

--- Try lexer-based matching for single-character delimiters
--- @param ctx table Context with bufnr, row, col, config, and delimiters
--- @return boolean Whether a match was found
function M.try_lexer_match(ctx)
  local bufnr = ctx.bufnr
  local row = ctx.row
  local col = ctx.col
  local config = ctx.config
  local delimiters = ctx.delimiters
  local create_highlight = ctx.create_highlight
  local store_marks = ctx.store_marks

  local line_matches = require('blink.pairs.rust').get_parsed_line(bufnr, row)
  if not line_matches or #line_matches == 0 then return false end

  local current_match = nil
  for i = 1, #line_matches do
    local match = line_matches[i]
    if match.col == col then
      current_match = match
      break
    end
  end

  local delimiter_utils = require('blink.pairs.delimiters')
  if not current_match or not delimiter_utils.is_delimiter(current_match.text, delimiters) then return false end

  local marks = {}

  local current_mark = create_highlight(bufnr, row, current_match.col, current_match.col + 1, config)
  table.insert(marks, current_mark)

  local matching_mark = nil

  if current_match.closing then
    local closing_text = current_match.closing
    local stack_height = current_match.stack_height

    if not delimiter_utils.is_delimiter(closing_text, delimiters, 'closing') then
      store_marks(bufnr, marks)
      return true
    end

    local line_count = vim.api.nvim_buf_line_count(bufnr)

    for search_row = row, line_count - 1 do
      local search_col = 0
      if search_row == row then search_col = current_match.col + 1 end

      local search_matches = require('blink.pairs.rust').get_parsed_line(bufnr, search_row)
      for i = 1, #search_matches do
        local match = search_matches[i]
        if match.col >= search_col then
          if not match.closing and match.text == closing_text and match.stack_height == stack_height then
            matching_mark = create_highlight(bufnr, search_row, match.col, match.col + 1, config)
            break
          end
        end
      end
      if matching_mark then break end
    end
  else
    local text = current_match.text
    local matching_text = delimiter_utils.get_matching(text, delimiters)

    if not matching_text then
      store_marks(bufnr, marks)
      return true
    end

    local stack_height = current_match.stack_height

    for search_row = row, 0, -1 do
      local search_matches = require('blink.pairs.rust').get_parsed_line(bufnr, search_row)
      if search_matches then
        for i = #search_matches, 1, -1 do
          local match = search_matches[i]
          if search_row < row or match.col < col then
            if match.closing and match.closing == text and match.stack_height == stack_height then
              matching_mark = create_highlight(bufnr, search_row, match.col, match.col + 1, config)
              break
            end
          end
        end
        if matching_mark then break end
      end
    end
  end

  if matching_mark then table.insert(marks, matching_mark) end

  store_marks(bufnr, marks)
  return true
end

--- Find matching closing delimiter
--- @param ctx table Context with bufnr, row, col, config, and delimiters
--- @param current_match table Current delimiter match
--- @return number|nil Mark ID if match found
function M.find_closing_match(ctx, current_match)
  local closing_text = current_match.closing
  local delimiter_utils = require('blink.pairs.delimiters')
  if not delimiter_utils.is_delimiter(closing_text, ctx.delimiters, 'closing') then return nil end

  local start_row = ctx.row
  local stack_height = current_match.stack_height
  local all_lines = vim.api.nvim_buf_get_lines(ctx.bufnr, 0, -1, false)

  for search_row = start_row, #all_lines - 1 do
    local search_col = 0
    if search_row == start_row then search_col = current_match.col + 1 end

    local search_matches = require('blink.pairs.rust').get_parsed_line(ctx.bufnr, search_row)
    if search_matches then
      for _, match in ipairs(search_matches) do
        if match.col >= search_col then
          if not match.closing and match.text == closing_text and match.stack_height == stack_height then
            return ctx.create_highlight(ctx.bufnr, search_row, match.col, match.col + 1, ctx.config)
          end
        end
      end
    end
  end

  return nil
end

--- Find matching opening delimiter
--- @param ctx table Context with bufnr, row, col, config, and delimiters
--- @param current_match table Current delimiter match
--- @return number|nil Mark ID if match found
function M.find_opening_match(ctx, current_match)
  local text = current_match.text
  if not require('blink.pairs.delimiters').get_matching(text, ctx.delimiters) then return nil end

  local row = ctx.row
  local col = ctx.col
  local stack_height = current_match.stack_height

  for search_row = row, 0, -1 do
    local search_matches = require('blink.pairs.rust').get_parsed_line(ctx.bufnr, search_row)
    if search_matches then
      for i = #search_matches, 1, -1 do
        local match = search_matches[i]
        if search_row < row or match.col < col then
          if match.closing and match.closing == text and match.stack_height == stack_height then
            return ctx.create_highlight(ctx.bufnr, search_row, match.col, match.col + 1, ctx.config)
          end
        end
      end
    end
  end

  return nil
end

--- Try multi-character delimiter matching with optimized handling
--- @param ctx table Context with bufnr, row, col, config, and delimiters
--- @return boolean Whether a match was found
function M.try_multi_char_match(ctx)
  local line = vim.api.nvim_get_current_line()
  if not line or line == '' then return false end

  local col = ctx.col
  local bufnr = ctx.bufnr
  local row = ctx.row
  local delimiters = ctx.delimiters

  local has_multi_char = false
  for opening, _ in pairs(delimiters.opening) do
    if #opening > 1 then
      has_multi_char = true
      break
    end
  end

  if not has_multi_char then return false end

  local create_highlight = ctx.create_highlight
  local store_marks = ctx.store_marks
  local config = ctx.config

  for opening, _ in pairs(delimiters.opening) do
    if #opening > 1 then
      local pos = 1
      local escaped_opening = vim.pesc(opening)

      while true do
        local start_pos = line:find(escaped_opening, pos)
        if not start_pos then break end

        local end_pos = start_pos + #opening - 1
        local start_col = start_pos - 1
        local end_col = end_pos - 1

        if col >= start_col and col <= end_col then
          local closing = delimiters.pairs[opening]
          if closing then
            local marks = {}
            table.insert(marks, create_highlight(bufnr, row, start_col, end_pos, config))

            local match_mark = M.find_matching_multi_char(ctx, start_col, opening, closing, false)
            if match_mark then
              table.insert(marks, match_mark)
              store_marks(bufnr, marks)
              return true
            end

            store_marks(bufnr, marks)
            return true
          end
        end

        pos = start_pos + 1
      end
    end
  end

  for closing, _ in pairs(delimiters.closing) do
    if #closing > 1 then
      local pos = 1
      local escaped_closing = vim.pesc(closing)

      while true do
        local start_pos = line:find(escaped_closing, pos)
        if not start_pos then break end

        local end_pos = start_pos + #closing - 1
        local start_col = start_pos - 1
        local end_col = end_pos - 1

        if col >= start_col and col <= end_col then
          local opening = delimiters.pairs[closing]
          if opening then
            local marks = {}
            table.insert(marks, create_highlight(bufnr, row, start_col, end_pos, config))

            local match_mark = M.find_matching_multi_char(ctx, start_col, closing, opening, true)
            if match_mark then
              table.insert(marks, match_mark)
              store_marks(bufnr, marks)
              return true
            end

            store_marks(bufnr, marks)
            return true
          end
        end

        pos = start_pos + 1
      end
    end
  end

  return false
end

--- Find matching delimiter for multi-character delimiters
--- @param ctx table Context information
--- @param start_col number Starting column
--- @param first string First delimiter (opening when searching forward, closing when searching backward)
--- @param second string Second delimiter (closing when searching forward, opening when searching backward)
--- @param reverse boolean If true, search backward for opening delimiter, otherwise search forward for closing delimiter
--- @return number|nil Mark ID if match found
function M.find_matching_multi_char(ctx, start_col, first, second, reverse)
  local bufnr = ctx.bufnr
  local config = ctx.config
  local create_highlight = ctx.create_highlight
  local first_len = #first
  local second_len = #second
  local same_delimiter = first == second

  local esc_first = vim.pesc(first)
  local esc_second = vim.pesc(second)

  local nesting_level = 1

  if reverse then
    for row = ctx.row, 0, -1 do
      local line = vim.api.nvim_buf_get_lines(bufnr, row, row + 1, false)[1]
      if not line or line == '' then goto continue end

      if row == ctx.row then
        if start_col < #line then line = line:sub(1, start_col) end
      end

      if #line == 0 then goto continue end

      local current_pos = #line + 1
      local sub_line = line

      while current_pos > 1 do
        local open_pos = rfind(sub_line, esc_second)
        local close_pos = rfind(sub_line, esc_first)

        if not open_pos and not close_pos then break end

        if same_delimiter and open_pos and close_pos and open_pos == close_pos then
          nesting_level = nesting_level - 1
          current_pos = open_pos
          sub_line = line:sub(1, current_pos - 1)

          if nesting_level == 0 then
            return create_highlight(bufnr, row, open_pos - 1, open_pos - 1 + second_len, config)
          end
        elseif close_pos and (not open_pos or close_pos > open_pos) then
          nesting_level = nesting_level + 1
          current_pos = close_pos
          sub_line = line:sub(1, current_pos - 1)
        elseif open_pos then
          nesting_level = nesting_level - 1
          current_pos = open_pos
          sub_line = line:sub(1, current_pos - 1)

          if nesting_level == 0 then
            return create_highlight(bufnr, row, open_pos - 1, open_pos - 1 + second_len, config)
          end
        end
      end

      ::continue::
    end
  else
    local search_start_col = start_col + first_len
    local all_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local total_lines = #all_lines

    for row = ctx.row, total_lines - 1 do
      local line = all_lines[row + 1]
      if not line or line == '' then goto continue end

      local col = 0

      if row == ctx.row then
        col = search_start_col
        if col >= #line then goto continue end
        line = line:sub(col + 1)
      end

      local current_pos = 0

      while current_pos < #line do
        local open_pos = line:find(esc_first, current_pos + 1)
        local close_pos = line:find(esc_second, current_pos + 1)

        if not open_pos and not close_pos then break end

        if same_delimiter and open_pos and close_pos and open_pos == close_pos then
          nesting_level = nesting_level - 1
          current_pos = close_pos

          if nesting_level == 0 then
            return create_highlight(bufnr, row, col + close_pos - 1, col + close_pos - 1 + second_len, config)
          end
        elseif open_pos and (not close_pos or open_pos < close_pos) then
          nesting_level = nesting_level + 1
          current_pos = open_pos
        elseif close_pos then
          nesting_level = nesting_level - 1
          current_pos = close_pos

          if nesting_level == 0 then
            return create_highlight(bufnr, row, col + close_pos - 1, col + close_pos - 1 + second_len, config)
          end
        end
      end

      ::continue::
    end
  end

  return nil
end

return M
