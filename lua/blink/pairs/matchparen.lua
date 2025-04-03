-- matchparen.lua - Handles highlighting of matching pairs
local M = {}

--- Initialize matchparen functionality
--- @param config blink.pairs.HighlightsConfig
function M.setup(config)
  if not (config.matchparen and config.matchparen.enabled) then return end

  local ns = vim.api.nvim_create_namespace('blink_pairs_matchparen')

  local extmarks = {}

  vim.api.nvim_create_autocmd({ 'CursorMoved', 'CursorMovedI' }, {
    group = vim.api.nvim_create_augroup('BlinkPairsMatchparen', {}),
    callback = function(ev)
      -- In insert mode, we'll get the CursorMovedI event, so we can ignore CursorMoved
      if vim.api.nvim_get_mode().mode:match('i') and ev.event == 'CursorMoved' then return end

      -- TODO: run this for all the windows
      local cursor = vim.api.nvim_win_get_cursor(0)
      local pair = require('blink.pairs.rust').get_match_pair(ev.buf, cursor[1] - 1, cursor[2])

      -- Clear extmarks
      if #extmarks > 0 then
        for _, extmark in ipairs(extmarks) do
          vim.api.nvim_buf_del_extmark(ev.buf, ns, extmark)
        end
        extmarks = {}
      end

      if pair == nil then return end

      -- Highlight matches
      for idx, match in ipairs(pair) do
        extmarks[idx] = vim.api.nvim_buf_set_extmark(ev.buf, ns, match.line, match.col, {
          end_col = match.col + match.text:len(),
          hl_group = config.matchparen.group,
          hl_mode = 'combine',
          priority = config.matchparen.priority,
        })
      end
    end,
  })
end

return M
