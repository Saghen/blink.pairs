--- @class blink.pairs.context.Treesitter
--- @field ctx blink.pairs.Context
local TS = {}
TS.__mt = { __index = TS }

--- @class MatchResult
--- @field ok boolean
--- @field matches boolean?

--- @param ctx blink.pairs.Context
--- @param query string
--- @param capture string
--- @return MatchResult
local function matches(ctx, query, capture)
  -- TODO: implementation
  return { ok = false }
end

--- @param self blink.pairs.context.Treesitter
--- @param query string
--- @return MatchResult
function TS:matches_whitelist(query)
  local key = ('matches_whitelist(%s)'):format(query)
  return require('blink.pairs.context.utils').memoize(
    self.ctx,
    key,
    function() return matches(self.ctx, query, 'pair') end
  )
end

--- @param self blink.pairs.context.Treesitter
--- @param query string
--- @return MatchResult
function TS:matches_blacklist(query)
  local key = ('matches_blacklist(%s)'):format(query)
  return require('blink.pairs.context.utils').memoize(
    self.ctx,
    key,
    function() return matches(self.ctx, query, 'nopair') end
  )
end

return TS
