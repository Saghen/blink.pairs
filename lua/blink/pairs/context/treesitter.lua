--- @class blink.pairs.context.Treesitter
--- @field ctx blink.pairs.Context
--- @field lang string? treesitter language at the context's cursor position
local TS = {
  --- @private
  --- @type table<string, string[]>
  __lang_to_ft = {
    angular = { 'htmlangular' },
    bash = { 'sh' },
    bibtex = { 'bib' },
    c_sharp = { 'cs' },
    commonlisp = { 'lisp' },
    cooklang = { 'cook' },
    devicetree = { 'dts' },
    eex = { 'elixir' },
    git_config = { 'gitconfig' },
    git_rebase = { 'gitrebase' },
    godot_resource = { 'gdresource' },
    idris = { 'idris2' },
    javascript = { 'javascriptreact' },
    javascript_glimmer = { 'javascript.glimmer' },
    latex = { 'tex' },
    linkerscript = { 'ld' },
    make = { 'automake' },
    markdown = { 'pandoc' },
    markdown_inline = { 'markdown' },
    muttrc = { 'neomuttrc' },
    poe_filter = { 'poefilter' },
    powershell = { 'ps1' },
    properties = { 'jproperties' },
    python = { 'gyp' },
    qmljs = { 'qml' },
    scala = { 'sbt' },
    slang = { 'shaderslang' },
    ssh_config = { 'sshconfig' },
    terraform = { 'terraform-vars' },
    starlark = { 'bzl' },
    systemverilog = { 'verilog' },
    tcl = { 'expect' },
    textproto = { 'pbtxt' },
    tsx = { 'typescriptreact', 'typescript.tsx' },
    typescript_glimmer = { 'typescript.glimmer' },
    udev = { 'udevrules' },
    xml = { 'svg', 'xsd', 'xslt' },
    xresources = { 'xdefaults' },
  },
}

--- @private
--- @type table<string, fun(ts: blink.pairs.context.Treesitter): ...>
TS.__field_constructors = {
  lang = function(ts)
    local ctx = ts.ctx
    local ok, parser = pcall(vim.treesitter.get_parser, ctx.bufnr)
    if not ok or not parser then return end
    local row, col = ctx.cursor.row - 1, ctx.cursor.col
    return parser:language_for_range({ row, col, row, col }):lang()
  end,
}

--- @private
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

--- @class blink.pairs.context.QueryResult
--- @field parser_found boolean true if a parser exists at the cursor position.
--- @field matches boolean

--- Search for a treesitter capture at the current position.
--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @param capture_name string
--- @return blink.pairs.context.QueryResult
function TS:matches_capture(query_name, capture_name)
  local ctx = self.ctx
  local key = ("matches_capture('%s', '%s')"):format(query_name, capture_name)
  return require('blink.pairs.context.utils').memoize(ctx, key, function()
    local ok, parser = pcall(vim.treesitter.get_parser, ctx.bufnr)
    if not ok or not parser then return { parser_found = false, matches = false } end

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
        local _, _, node_row_end, node_col_end = node:range()
        local inside = vim.treesitter.is_in_node_range(node, row, col)
        local after = node_row_end == row and node_col_end == ctx.prev_non_ws_col
        if
          (capture == capture_name .. '.inside' and inside)
          or (capture == capture_name .. '.inside_or_after' and (inside or after))
          or (capture == capture_name .. '.after' and after)
        then
          matches = true
          return
        end
      end
    end)
    return { parser_found = true, matches = matches }
  end)
end

--- Return a `blink.pairs.context.QueryResult` indicating whether the “pair”
--- capture matches at the current Treesitter node.
---
--- This implements a whitelist. Only positions with an explicit “pair” capture
--- pass: `matches` is true only if `parser_found` is true and at least one
--- “pair” capture was found.
--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @return blink.pairs.context.QueryResult
function TS:whitelist(query_name)
  local result = self:matches_capture(query_name, 'pair')
  return { parser_found = result.parser_found, matches = result.parser_found and result.matches }
end

--- Return a `blink.pairs.context.QueryResult` indicating whether the “nopair”
--- capture does *not* match at the current Treesitter node.
---
--- This implements a blacklist. Positions with a “nopair” capture are
--- excluded: `matches` is true if either no parser is found or the “nopair”
--- capture yields no matches.
--- @param self blink.pairs.context.Treesitter
--- @param query_name string
--- @return blink.pairs.context.QueryResult
function TS:blacklist(query_name)
  local result = self:matches_capture(query_name, 'nopair')
  return { parser_found = result.parser_found, matches = not (result.parser_found and result.matches) }
end

--- Returns the language names to be used when loading parsers for `filetypes`.
--- @see vim.treesitter.language.get_filetypes
--- @param filetypes string | string[]
--- @return string[]
function TS.get_langs(filetypes)
  filetypes = type(filetypes) == 'table' and filetypes or { filetypes }
  ---@cast filetypes string[]

  local r = {}
  local seen = {}

  for lang, fts in pairs(TS.__lang_to_ft) do
    if not seen[lang] then
      for _, ft in ipairs(fts) do
        if vim.tbl_contains(filetypes, ft) then
          r[#r + 1] = lang
          seen[lang] = true
          break
        end
      end
    end
  end

  for _, ft in ipairs(filetypes) do
    local lang = vim.treesitter.language.get_lang(ft)
    if lang and not seen[lang] then
      r[#r + 1] = lang
      seen[lang] = true
    end
  end

  return r
end

--- Returns the filetypes for which each parser from `langs` is used. The list
--- includes `lang` itself.
--- @see vim.treesitter.language.get_filetypes
--- @param langs string | string[]
--- @return string[]
function TS.get_filetypes(langs)
  langs = type(langs) == 'table' and langs or { langs }
  ---@cast langs string[]

  local r = {}
  local seen = {}

  for _, lang in ipairs(langs) do
    r[#r + 1] = lang
    seen[lang] = true
  end

  for lang, fts in pairs(TS.__lang_to_ft) do
    if vim.tbl_contains(langs, lang) then
      for _, ft in ipairs(fts) do
        if not seen[ft] then
          r[#r + 1] = ft
          seen[ft] = true
        end
      end
    end
  end

  for _, lang in ipairs(langs) do
    for _, ft in ipairs(vim.treesitter.language.get_filetypes(lang)) do
      if not seen[ft] then
        r[#r + 1] = ft
        seen[ft] = true
      end
    end
  end

  return r
end

--- Checks if a given treesitter language is found at the cursor position. If
--- no treesitter language is found for the current cursor position (i.e. the
--- relevant treesitter parser is not installed), then fall back to using
--- the filetype.
---
--- `langs` may include both Treesitter languages or vim filetypes. Each entry
--- is normalized to both representations and checked agains the current parser
--- or filetype.
---
--- @param self blink.pairs.context.Treesitter
--- @param langs string | string[]
function TS:is_language(langs)
  langs = type(langs) == 'table' and langs or { langs }
  if self.lang ~= nil then
    return vim.tbl_contains(langs, self.lang) or vim.tbl_contains(TS.get_langs(langs), self.lang)
  else
    return vim.tbl_contains(langs, self.ctx.ft) or vim.tbl_contains(TS.get_filetypes(langs), self.ctx.ft)
  end
end

return TS
