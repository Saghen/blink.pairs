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
      ['!'] = { { '<!--', '-->', languages = { 'html', 'markdown' } } },
      ['('] = ')',
      ['['] = ']',
      ['{'] = '}',
      ["'"] = {
        {
          "'''",
          "'''",
          when = function(ctx) return ctx:text_before_cursor(2) == "''" end,
          languages = { 'python' },
        },
        {
          "'",
          enter = false,
          space = false,
          when = function(ctx)
            if ctx.treesitter:is_any_lang_or_ft({ 'bib', 'tex', 'plaintex' }) or ctx.char_under_cursor:match('%w') then
              return false
            end

            return ctx.treesitter:blacklist('singlequote').matches
          end,
        },
      },
      ['"'] = {
        { 'r#"', '"#', languages = { 'rust' }, priority = 100 },
        {
          '"""',
          '"""',
          when = function(ctx) return ctx:text_before_cursor(2) == '""' end,
          languages = { 'python', 'elixir', 'julia', 'kotlin', 'scala', 'sbt' },
        },
        { '"', enter = false, space = false },
      },
      ['`'] = {
        {
          '```',
          '```',
          when = function(ctx) return ctx:text_before_cursor(2) == '``' end,
          languages = { 'markdown', 'vimwiki', 'rmarkdown', 'rmd', 'pandoc', 'quarto', 'typst' },
        },
        { '`', "'", languages = { 'bib', 'tex', 'plaintex' } },
        { '`', enter = false, space = false },
      },
      ['_'] = {
        {
          '_',
          when = function(ctx)
            local rule = require('blink.pairs.rule')

            if rule.is_in_span('math') then return false end
            if ctx.treesitter:is_lang_or_ft('markdown') then return not ctx.char_under_cursor:match('%w') end

            return true
          end,
          languages = { 'markdown', 'typst' },
        },
      },
      ['<'] = {
        {
          '<',
          '>',
          when = function(ctx) return ctx.treesitter:whitelist('angle').matches end,
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
      languages = { def.languages, { 'table', 'nil' } },
      when = { def.when, { 'function', 'nil' } },
      enter = { def.enter, { 'boolean', 'function', 'nil' } },
      backspace = { def.backspace, { 'boolean', 'function', 'nil' } },
      space = { def.space, { 'boolean', 'function', 'nil' } },
    }, def)
  end
end

return mappings
