--- @class (exact) blink.pairs.HighlightsConfig
--- @field enabled boolean
--- @field cmdline boolean Requires `require('vim._extui').enable({})`
--- @field groups string[] Highlight groups for matched pairs, in order that they'll appear based on depth
--- @field unmatched_group string Highlight group for unmatched pairs
--- @field priority number
--- @field ns integer
--- @field matchparen blink.pairs.MatchparenConfig

--- @class (exact) blink.pairs.MatchparenConfig
--- @field enabled boolean
--- @field cmdline boolean Requires `require('vim._extui').enable({})`. Disabled by default due to only showing matchparen when moving the cursor, and not when typing.
--- @field group string Highlight group for the matching pair
--- @field priority number Priority of the highlight

local validate = require('blink.pairs.config.utils').validate
local highlights = {
  --- @type blink.pairs.HighlightsConfig
  default = {
    enabled = true,
    cmdline = true,
    groups = {
      'BlinkPairsOrange',
      'BlinkPairsPurple',
      'BlinkPairsBlue',
    },
    unmatched_group = 'BlinkPairsUnmatched',
    priority = 200,
    ns = vim.api.nvim_create_namespace('blink.pairs'),
    matchparen = {
      enabled = true,
      cmdline = false,
      group = 'MatchParen',
      priority = 250,
    },
  },
}

function highlights.validate(config)
  validate('highlights', {
    enabled = { config.enabled, 'boolean' },
    cmdline = { config.cmdline, 'boolean' },
    unmatched_group = { config.unmatched_group, 'string' },
    groups = { config.groups, 'table' },
    priority = { config.priority, 'number' },
    ns = { config.ns, 'number' },
    matchparen = { config.matchparen, 'table', true },
  }, config)

  validate('highlights.matchparen', {
    enabled = { config.matchparen.enabled, 'boolean' },
    cmdline = { config.cmdline, 'boolean' },
    group = { config.matchparen.group, 'string' },
    priority = { config.matchparen.priority, 'number' },
  }, config.matchparen)
end

return highlights
