local highlighter = {}

--- @param config blink.pairs.HighlightsConfig
function highlighter.register(config)
  vim.api.nvim_set_decoration_provider(config.ns, {
    on_win = function(_, _, bufnr) return require('blink.pairs.watcher').attach(bufnr) end,
    on_line = function(_, _, bufnr, line_number)
      for _, match in ipairs(require('blink.pairs.rust').get_line_matches(bufnr, line_number)) do
        if line_number > 5 and line_number < 9 then match.line = line_number           -- vim.print(match)
end
        vim.api.nvim_buf_set_extmark(bufnr, config.ns, line_number, match.col, {
          end_col = match.col + match.text:len(),
          hl_group = config.groups[match.stack_height % #config.groups + 1],
          hl_mode = 'combine',
          priority = config.priority,
          ephemeral = true,
        })
      end
    end,
  })

  if config.matchparen.enabled then require('blink.pairs.matchparen').setup(config) end
end

return highlighter
