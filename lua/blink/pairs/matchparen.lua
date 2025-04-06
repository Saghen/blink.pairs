-- matchparen.lua - Handles highlighting of matching pairs
local M = {}

--- Initialize matchparen functionality
--- @param config blink.pairs.HighlightsConfig
function M.setup(config)
  if not (config.matchparen and config.matchparen.enabled) then return end

  local ns = vim.api.nvim_create_namespace('blink_pairs_matchparen')
  local last_buf

  vim.api.nvim_create_autocmd({ 'CursorMoved', 'CursorMovedI' }, {
    group = vim.api.nvim_create_augroup('BlinkPairsMatchparen', {}),
    callback = function(ev)
      -- In insert mode, we'll get the CursorMovedI event, so we can ignore CursorMoved
      if vim.api.nvim_get_mode().mode:match('i') and ev.event == 'CursorMoved' then return end

      -- TODO: run this for all the windows
      local cursor = vim.api.nvim_win_get_cursor(0)
      local buf = vim.api.nvim_get_current_buf()
      local pair = require('blink.pairs.rust').get_match_pair(buf, cursor[1] - 1, cursor[2])

      -- Clear extmarks
      if last_buf and vim.api.nvim_buf_is_valid(last_buf) then vim.api.nvim_buf_clear_namespace(last_buf, ns, 0, -1) end
      last_buf = buf

      if pair == nil then return end

      -- Highlight matches
      for i, match in ipairs(pair) do
        vim.api.nvim_buf_set_extmark(buf, ns, match.line, match.col, {
          end_col = match.col + match[i]:len(),
          hl_group = config.matchparen.group,
          hl_mode = 'combine',
          priority = config.matchparen.priority,
        })
      end
    end,
  })
end

return M
