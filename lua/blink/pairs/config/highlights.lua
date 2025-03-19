--- @class (exact) blink.pairs.HighlightsConfig
--- @field enabled boolean
--- @field groups string[]
--- @field priority number
--- @field ns integer

local validate = require('blink.cmp.config.utils').validate
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
  },
}

function highlights.validate(config)
  validate('highlights', {
    enabled = { config.enabled, 'boolean' },
    groups = { config.groups, 'table' },
    priority = { config.priority, 'number' },
    ns = { config.ns, 'number' },
  }, config)
end

return highlights
