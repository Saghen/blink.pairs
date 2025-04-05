--- @class (exact) blink.pairs.MappingsConfig
--- @field enabled boolean
--- @field pairs blink.pairs.RuleDefinitions

local validate = require('blink.pairs.config.utils').validate
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
          space = false,
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
        { '"', enter = false, space = false },
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
        { '`', enter = false, space = false },
      },
      ['_'] = {
        {
          '_',
          when = function()
            local rule = require('blink.pairs.rule')
            local in_math = rule.is_in_span('math')

            if in_math then return false end

            return true
          end,
          filetypes = { 'markdown', 'typst' },
        },
      },
    },
  },
}

function mappings.validate(config)
  validate('mappings', {
    enabled = { config.enabled, 'boolean' },
    pairs = { config.pairs, 'table' },
  }, config)

  for key, defs in pairs(config.pairs) do
    mappings.validate_rules(key, defs)
  end
end

function mappings.validate_rules(key, defs)
  if type(defs) == 'string' then return end

  if not vim.islist(defs) then defs = { defs } end

  for i, def in ipairs(defs) do
    validate('mappings.pairs.[' .. key .. '].' .. i, {
      [1] = { def[1], 'string' },
      [2] = { def[2], { 'string', 'nil' } },
      priority = { def.priority, { 'number', 'nil' } },
      filetypes = { def.filetypes, { 'table', 'nil' } },
      when = { def.when, { 'function', 'nil' } },
      enter = { def.enter, { 'boolean', 'function', 'nil' } },
      backspace = { def.backspace, { 'boolean', 'function', 'nil' } },
      space = { def.space, { 'boolean', 'function', 'nil' } },
    }, def)
  end
end

return mappings
