local highlighter = {}

--- @param config blink.pairs.HighlightsConfig
function highlighter.register(config)
  vim.api.nvim_set_decoration_provider(config.ns, {
    on_win = function(_, _, bufnr)
      if not config.cmdline and vim.api.nvim_get_mode().mode:match('c') then return false end

      vim.api.nvim_buf_clear_namespace(bufnr, config.ns, 0, -1)
      return require('blink.pairs.watcher').attach(bufnr)
    end,
    on_line = function(_, _, bufnr, line_number)
      for _, match in ipairs(require('blink.pairs.rust').get_line_matches(bufnr, line_number)) do
        local hl_group = match.stack_height == nil and config.unmatched_group
          or config.groups[match.stack_height % #config.groups + 1]

        vim.api.nvim_buf_set_extmark(bufnr, config.ns, line_number, match.col, {
          end_col = match.col + match[1]:len(),
          hl_group = hl_group,
          hl_mode = 'combine',
          priority = config.priority,
        })
      end
    end,
  })

  if config.matchparen.enabled then require('blink.pairs.matchparen').setup(config) end
end

return highlighter
