--- @class (exact) blink.pairs.HighlightsConfig
--- @field enabled boolean
--- @field groups string[]
--- @field priority number
--- @field ns integer
--- @field matchparen blink.pairs.MatchparenConfig

--- @class (exact) blink.pairs.MatchparenConfig
--- @field enabled boolean
--- @field group string Highlight group for the matching pair
--- @field priority number Priority of the highlight

local validate = require('blink.pairs.config.utils').validate
local highlights = {
  --- @type blink.pairs.HighlightsConfig
  default = {
    enabled = true,
    groups = {
      'BlinkPairsOrange',
      'BlinkPairsPurple',
      'BlinkPairsBlue',
    },
    priority = 200,
    ns = vim.api.nvim_create_namespace('blink.pairs'),
    matchparen = {
      enabled = true,
      group = 'MatchParen',
      priority = 250,
    },
  },
}

function highlights.validate(config)
  validate('highlights', {
    enabled = { config.enabled, 'boolean' },
    groups = { config.groups, 'table' },
    priority = { config.priority, 'number' },
    ns = { config.ns, 'number' },
    matchparen = { config.matchparen, 'table', true },
  }, config)

  validate('highlights.matchparen', {
    enabled = { config.matchparen.enabled, 'boolean' },
    group = { config.matchparen.group, 'string' },
    priority = { config.matchparen.priority, 'number' },
  }, config.matchparen)
end

return highlights
