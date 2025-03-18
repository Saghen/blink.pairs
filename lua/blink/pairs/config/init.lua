--- @class (exact) blink.pairs.ConfigStrict
--- @field mappings blink.pairs.MappingsConfig
--- @field highlights blink.pairs.HighlightsConfig
--- @field debug boolean

--- @type blink.pairs.ConfigStrict
local config = {
  mappings = require('blink.pairs.config.mappings').default,
  highlights = require('blink.pairs.config.highlighter').default,
  debug = false,
}

--- @type blink.pairs.ConfigStrict
--- @diagnostic disable-next-line: missing-fields
local M = {}

--- @param config blink.pairs.ConfigStrict
function M.validate(config)
  -- use vim.validate to validate the config
end

--- @param user_config blink.pairs.Config
function M.merge_with(user_config)
  config = vim.tbl_deep_extend('force', config, user_config)
  M.validate(config)
end

return setmetatable(M, {
  __index = function(_, k) return config[k] end,
})
