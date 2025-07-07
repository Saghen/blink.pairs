local M = {}

--- Memoize the result of a function for a context with a given key.
--- @generic T
--- @param ctx blink.pairs.Context
--- @param key string
--- @param fn fun():T
--- @return T
function M.memoize(ctx, key, fn)
  local cache = rawget(ctx, '__cache') or {}
  rawset(ctx, '__cache', cache)

  if cache[key] ~= nil then return cache[key] end

  local result = fn()
  cache[key] = result
  return result
end

return M
