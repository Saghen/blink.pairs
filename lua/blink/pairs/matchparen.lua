-- matchparen.lua - Handles highlighting of matching pairs
local M = {}

--- Initialize matchparen functionality
--- @param config blink.pairs.HighlightsConfig
function M.setup(config)
  if not (config.matchparen and config.matchparen.enabled) then return end

  local ns = vim.api.nvim_create_namespace('blink_pairs_matchparen')
  local last_buf

  ---@type vim.api.keyset.events[]
  local autocmds = { 'CursorMoved', 'CursorMovedI' }
  if vim.fn.exists('##CursorMovedC') == 1 then autocmds[#autocmds + 1] = 'CursorMovedC' end
  -- luals bug https://github.com/LuaLS/lua-language-server/issues/3229
  ---@diagnostic disable-next-line: param-type-mismatch
  vim.api.nvim_create_autocmd(autocmds, {
    group = vim.api.nvim_create_augroup('BlinkPairsMatchparen', {}),
    callback = function(ev)
      local mode = vim.api.nvim_get_mode().mode
      -- In insert mode, we'll get the CursorMovedI event, so we can ignore CursorMoved
      if mode:match('i') and ev.event ~= 'CursorMovedI' or mode:match('c') and (ev.event ~= 'CursorMovedC') then
        return
      end

      -- TODO: run this for all the windows
      local ctx = require('blink.pairs.context').new()
      -- prompt mark (`:`, `/`) is not considered when do parsing
      local prompt_len = (mode:match('c') and 1 or 0)
      local cursor = { ctx.cursor.row, ctx.cursor.col + prompt_len }
      local buf = ctx.bufnr
      local pair = require('blink.pairs.rust').get_match_pair(buf, cursor[1] - 1, cursor[2])

      -- Clear extmarks
      if last_buf and vim.api.nvim_buf_is_valid(last_buf) then vim.api.nvim_buf_clear_namespace(last_buf, ns, 0, -1) end
      last_buf = buf

      if pair == nil then return end

      -- Highlight matches
      for i, match in ipairs(pair) do
        vim.api.nvim_buf_set_extmark(buf, ns, match.line, match.col, {
          end_col = match.col + (match[i] or match[1]):len(),
          hl_group = config.matchparen.group,
          hl_mode = 'combine',
          priority = config.matchparen.priority,
        })
      end
    end,
  })
end

return M
