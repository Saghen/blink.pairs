-- matchparen.lua - Handles highlighting of matching pairs
local M = {}

local ns = nil
local marks_by_buf = {}

--- Initialize matchparen functionality
--- @param config blink.pairs.HighlightsConfig
function M.setup(config)
  if not (config.matchparen and config.matchparen.enabled) then return end

  ns = vim.api.nvim_create_namespace('blink_pairs_matchparen')

  local augroup = vim.api.nvim_create_augroup('BlinkPairsMatchparen', { clear = true })

  vim.api.nvim_create_autocmd({ 'CursorMoved', 'CursorMovedI' }, {
    group = augroup,
    callback = function() M.update_highlights(config) end,
  })

  vim.api.nvim_create_autocmd('BufLeave', {
    group = augroup,
    callback = function()
      local bufnr = vim.api.nvim_get_current_buf()
      M.clear_highlights(bufnr)
    end,
  })
end

--- Clear existing highlights for a buffer
--- @param bufnr number Buffer number
function M.clear_highlights(bufnr)
  if not ns or not marks_by_buf[bufnr] then return end

  for _, mark_id in ipairs(marks_by_buf[bufnr] or {}) do
    pcall(vim.api.nvim_buf_del_extmark, bufnr, ns, mark_id)
  end

  marks_by_buf[bufnr] = nil
end

--- Update matching pair highlights based on cursor position
--- @param config blink.pairs.HighlightsConfig
function M.update_highlights(config)
  local bufnr = vim.api.nvim_get_current_buf()

  if not (config and config.matchparen and config.matchparen.enabled) then return end

  if ns and marks_by_buf[bufnr] then
    for _, mark_id in ipairs(marks_by_buf[bufnr]) do
      pcall(vim.api.nvim_buf_del_extmark, bufnr, ns, mark_id)
    end
  end

  marks_by_buf[bufnr] = {}

  local delimiter_module = require('blink.pairs.delimiters')
  local delimiters = delimiter_module.get_active_delimiters()
  if not delimiters then return end

  local current_ft = vim.bo.filetype

  local global_config = require('blink.pairs.config')
  if global_config and global_config.mappings and global_config.mappings.enabled then
    local pairs_config = global_config.mappings.pairs
    if pairs_config then
      for _, rules in pairs(pairs_config) do
        if type(rules) == 'table' then
          local rules_list = vim.islist(rules) and rules or { rules }

          for _, rule in ipairs(rules_list) do
            if type(rule) == 'table' and rule[1] and rule[2] and #rule[1] > 1 then
              local applies = true
              if rule.filetypes then
                applies = false
                for _, ft in ipairs(rule.filetypes) do
                  if ft == current_ft then
                    applies = true
                    break
                  end
                end
              end

              if applies and not delimiters.opening[rule[1]] then
                delimiters.opening[rule[1]] = true
                delimiters.closing[rule[2]] = true
                delimiters.pairs[rule[1]] = rule[2]
                delimiters.pairs[rule[2]] = rule[1]
              end
            end
          end
        end
      end
    end
  end

  local cursor = vim.api.nvim_win_get_cursor(0)
  local row, col = cursor[1] - 1, cursor[2]

  local highlight_fn = function(row, col, end_col)
    return vim.api.nvim_buf_set_extmark(bufnr, ns, row, col, {
      end_col = end_col,
      hl_group = config.matchparen.group,
      priority = config.matchparen.priority,
    })
  end

  local ctx = {
    bufnr = bufnr,
    row = row,
    col = col,
    config = config,
    delimiters = delimiters,
    create_highlight = function(bufnr, row, col, end_col)
      local mark_id = highlight_fn(row, col, end_col)
      return mark_id
    end,
    store_marks = function(_, marks) marks_by_buf[bufnr] = marks end,
    rules = delimiter_module.get_active_rules(),
  }

  local matcher = require('blink.pairs.matcher')
  local matched = false

  if config.matchparen.treesitter then matched = matcher.try_treesitter_match(ctx) end

  if not matched then matched = matcher.try_lexer_match(ctx) end

  if not matched then matcher.try_multi_char_match(ctx) end
end

function M.get_namespace() return ns end

return M
