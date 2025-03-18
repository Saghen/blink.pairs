--- @class (exact) blink.pairs.HighlightsConfig
--- @field enabled boolean
--- @field groups string[]
--- @field priority number
--- @field ns integer

local highlights = {
  --- @type blink.pairs.HighlightsConfig
  default = {
    enabled = true,
    groups = {
      'RainbowOrange',
      'RainbowPurple',
      'RainbowBlue',
    },
    priority = 200,
    ns = vim.api.nvim_create_namespace('blink.pairs'),
  },
}

function highlights.validate(config) end

return highlights
