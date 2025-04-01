local highlighter = {}

--- @class blink.pairs.MatchparenConfig
--- @field enabled boolean
--- @field group string Highlight group for the matching pair
--- @field priority number Priority of the highlight
--- @field treesitter boolean Whether to use treesitter for matching pairs

--- @param config blink.pairs.HighlightsConfig
function highlighter.register(config)
  vim.api.nvim_set_decoration_provider(config.ns, {
    on_win = function(_, _, bufnr) return require('blink.pairs.watcher').attach(bufnr) end,
    on_line = function(_, _, bufnr, line_number)
      for _, match in ipairs(require('blink.pairs.rust').get_parsed_line(bufnr, line_number)) do
        vim.api.nvim_buf_set_extmark(bufnr, config.ns, line_number, match.col, {
          end_col = match.col + 1,
          hl_group = config.groups[match.stack_height % #config.groups + 1],
          hl_mode = 'combine',
          priority = config.priority,
          ephemeral = true,
        })
      end
    end,
  })

  if config.matchparen and config.matchparen.enabled then require('blink.pairs.matchparen').setup(config) end
end

return highlighter
