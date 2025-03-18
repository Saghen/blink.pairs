--- @class (exact) blink.pairs.MappingsConfig
--- @field enabled boolean
--- @field pairs blink.pairs.RuleDefinitions

local mappings = {
  --- @type blink.pairs.MappingsConfig
  default = {
    enabled = true,
    pairs = {
      -- TODO: the `when` clauses should receive a stdlib
      ['!'] = { { '<!--', '-->', filetypes = { 'html', 'markdown' } } },
      ['('] = ')',
      ['['] = ']',
      ['{'] = '}',
      ["'"] = {
        {
          "'''",
          "'''",
          when = function()
            local cursor = vim.api.nvim_win_get_cursor(0)
            local line = vim.api.nvim_get_current_line()
            return line:sub(cursor[2] - 1, cursor[2]) == "''"
          end,
          filetypes = { 'python' },
        },
        {
          "'",
          enter = false,
          when = function()
            local cursor = vim.api.nvim_win_get_cursor(0)
            local char = vim.api.nvim_get_current_line():sub(cursor[2], cursor[2])
            return not char:match('%w')
          end,
        },
      },
      ['"'] = {
        { 'r#"', '"#', filetypes = { 'rust' }, priority = 100 },
        {
          '"""',
          '"""',
          when = function()
            local cursor = vim.api.nvim_win_get_cursor(0)
            local line = vim.api.nvim_get_current_line()
            return line:sub(cursor[2] - 1, cursor[2]) == '""'
          end,
          filetypes = { 'python', 'elixir', 'julia', 'kotlin', 'scala', 'sbt' },
        },
        { '"', enter = false },
      },
      ['`'] = {
        {
          '```',
          '```',
          when = function()
            local cursor = vim.api.nvim_win_get_cursor(0)
            local line = vim.api.nvim_get_current_line()
            return line:sub(cursor[2] - 1, cursor[2]) == '``'
          end,
          filetypes = { 'markdown', 'vimwiki', 'rmarkdown', 'rmd', 'pandoc', 'quarto', 'typst' },
        },
        { '`', enter = false },
      },
    },
  },
}

function mappings.validate(config) end

return mappings
