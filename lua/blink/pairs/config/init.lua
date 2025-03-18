--- @class (exact) blink.pairs.ConfigStrict
--- @field mappings blink.pairs.MappingsConfig
--- @field highlights blink.pairs.HighlightsConfig
--- @field debug boolean

local validate = require('blink.pairs.config.utils').validate
--- @type blink.pairs.ConfigStrict
local config = {
  mappings = require('blink.pairs.config.mappings').default,
  highlights = require('blink.pairs.config.highlights').default,
  debug = false,
}

--- @type blink.pairs.ConfigStrict
--- @diagnostic disable-next-line: missing-fields
local M = {}

--- @param cfg blink.pairs.ConfigStrict
function M.validate(cfg)
  validate('config', {
    mappings = { cfg.mappings, 'table' },
    highlights = { cfg.highlights, 'table' },
    debug = { cfg.debug, 'boolean' },
  }, cfg)

  require('blink.pairs.config.mappings').validate(cfg.mappings)
  require('blink.pairs.config.highlights').validate(cfg.highlights)
end

--- @param user_config blink.pairs.Config
function M.merge_with(user_config)
  config = vim.tbl_deep_extend('force', config, user_config)
  M.validate(config)
end

return setmetatable(M, {
  __index = function(_, k) return config[k] end,
})
