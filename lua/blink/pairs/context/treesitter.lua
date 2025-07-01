--- @class blink.pairs.context.Treesitter
--- @field ctx blink.pairs.Context
--- @field lang string
local TS = {}
---@type table<string, fun(ts: blink.pairs.context.Treesitter): ...>
TS.__field_constructors = {
  lang = function(ts)
    local ctx = ts.ctx
    local ok, parser = pcall(vim.treesitter.get_parser, ctx.bufnr)
    if not ok or not parser then return vim.treesitter.language.get_lang(ctx.ft) end
    local row, col = ctx.cursor.row - 1, ctx.cursor.col
    return parser:language_for_range({ row, col, row, col + 1 }):lang()
  end,
}
TS.__mt = {
  __index = function(ts, key)
    if TS[key] ~= nil then
      return TS[key]
    elseif TS.__field_constructors[key] ~= nil then
      local value = TS.__field_constructors[key](ts)
      rawset(ts, key, value)
      return value
    end
  end,
}

--- @class MatchResult
--- @field ok boolean
--- @field matches boolean

--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @param capture_name string
--- @return MatchResult
function TS:matches_capture(query_name, capture_name)
  local ctx = self.ctx
  local key = ("matches_capture('%s', '%s')"):format(query_name, capture_name)
  return require('blink.pairs.context.utils').memoize(ctx, key, function()
    local ok, parser = pcall(vim.treesitter.get_parser, ctx.bufnr)
    if not ok or not parser then return { ok = false, matches = false } end

    local row, col = ctx.cursor.row - 1, ctx.cursor.col

    local matches = false
    parser:for_each_tree(function(tree, ltree)
      if matches then
        -- a match has already been found
        return
      end

      local root = tree:root()
      local root_row_start, _, root_row_end, _ = root:range()
      if root_row_start > row or root_row_end < row then return end

      local query = vim.treesitter.query.get(ltree:lang(), 'pairs.' .. query_name)
      if not query then return end

      for id, node in query:iter_captures(root, 0, row, row + 1) do
        local capture = query.captures[id]
        local _, node_start, _, node_end = node:range()
        if
          (capture == capture_name .. '.inside' and node_start < col and col < node_end)
          or (capture == capture_name .. '.inside_or_after' and node_start < ctx.prev_non_ws_col and ctx.prev_non_ws_col <= node_end)
          or (capture == capture_name .. '.after' and node_end == ctx.prev_non_ws_col)
        then
          matches = true
          return
        end
      end
    end)
    return { ok = true, matches = matches }
  end)
end

--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @return MatchResult
function TS:whitelist(query_name)
  local result = self:matches_capture(query_name, 'pair')
  return { ok = result.ok, matches = result.ok and result.matches }
end

--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @return MatchResult
function TS:blacklist(query_name)
  local result = self:matches_capture(query_name, 'nopair')
  return { ok = result.ok, matches = not (result.ok and result.matches) }
end

--- @param self blink.pairs.context.Treesitter
--- @param lang_or_ft string
function TS:is_lang_or_ft(lang_or_ft)
  return lang_or_ft == self.lang or vim.treesitter.language.get_lang(lang_or_ft) == self.lang
end

--- @param self blink.pairs.context.Treesitter
--- @param langs_or_fts string[]
function TS:is_any_lang_or_ft(langs_or_fts)
  return vim.iter(langs_or_fts):any(function(lang_or_ft) return self:is_lang_or_ft(lang_or_ft) end)
end

return TS
