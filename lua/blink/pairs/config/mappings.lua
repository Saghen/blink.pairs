--- @class (exact) blink.pairs.MappingsConfig
--- @field enabled boolean
--- @field disabled_filetypes string[]
--- @field pairs blink.pairs.RuleDefinitions

local validate = require('blink.pairs.config.utils').validate
local mappings = {
  --- @type blink.pairs.MappingsConfig
  default = {
    enabled = true,
    disabled_filetypes = {},
    pairs = {
      ['!'] = { { '<!--', '-->', filetypes = { 'html', 'markdown' } } },
      ['('] = ')',
      ['['] = ']',
      ['{'] = '}',
      ["'"] = {
        {
          "'''",
          "'''",
          when = function(ctx) return ctx:text_before_cursor(2) == "''" end,
          filetypes = { 'python' },
        },
        {
          "'",
          enter = false,
          space = false,
          when = function(ctx)
            if vim.list_contains({ 'bib', 'tex', 'plaintex' }, ctx.ft) or ctx.char_under_cursor:match('%w') then
              return false
            end

            return ctx.treesitter:matches_blacklist('singlequote').should_pair
          end,
        },
      },
      ['"'] = {
        { 'r#"', '"#', filetypes = { 'rust' }, priority = 100 },
        {
          '"""',
          '"""',
          when = function(ctx) return ctx:text_before_cursor(2) == '""' end,
          filetypes = { 'python', 'elixir', 'julia', 'kotlin', 'scala', 'sbt' },
        },
        { '"', enter = false, space = false },
      },
      ['`'] = {
        {
          '```',
          '```',
          when = function(ctx) return ctx:text_before_cursor(2) == '``' end,
          filetypes = { 'markdown', 'vimwiki', 'rmarkdown', 'rmd', 'pandoc', 'quarto', 'typst' },
        },
        { '`', "'", filetypes = { 'bib', 'tex', 'plaintex' } },
        { '`', enter = false, space = false },
      },
      ['_'] = {
        {
          '_',
          when = function(ctx)
            local rule = require('blink.pairs.rule')

            if rule.is_in_span('math') then return false end
            if ctx.ft == 'markdown' then return not ctx.char_under_cursor:match('%w') end

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
    disabled_filetypes = { config.disabled_filetypes, 'table' },
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
