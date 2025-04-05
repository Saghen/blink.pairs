--- @class blink.pairs.Rule
--- @field priority number
--- @field opening string
--- @field closing string
--- @field when fun(): boolean
--- @field enter fun(): boolean
--- @field backspace fun(): boolean
--- @field space fun(): boolean

--- @alias blink.pairs.RulesByKey table<string, blink.pairs.Rule[]>

--- @alias blink.pairs.Mode 'enter' | 'backspace' | 'space'

local M = {}

--- @generic T
--- @param val T | fun(): T
--- @param default? T
--- @return fun(): T
local function as_func(val, default)
  if type(val) == 'function' then return val end
  if val == nil then
    return function() return default end
  end
  return function() return val end
end

--- Helper function to safely check if we're inside a specific span type
--- @param span_name string The name of the span to check for (e.g., "math")
--- @return boolean Whether we're currently inside the specified span
function M.is_in_span(span_name)
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor = vim.api.nvim_win_get_cursor(0)

  local span_at = require('blink.pairs.rust').get_span_at(bufnr, cursor[1] - 1, cursor[2])
  return span_at == span_name
end

--- Takes a table of user friendly rule definitions and converts it to a table of rules
--- @param definitions blink.pairs.RuleDefinitions
--- @return blink.pairs.RulesByKey
function M.parse(definitions)
  --- @type blink.pairs.RulesByKey
  local rules = {}

  for key, defs in pairs(definitions) do
    if type(defs) ~= 'table' or not vim.islist(defs) then defs = { defs } end
    --- @cast defs (blink.pairs.RuleDefinition | string)[]

    for _, def in ipairs(defs) do
      local rule = M.rule_from_def(key, def)

      -- Opening key
      if rules[key] == nil then rules[key] = {} end
      table.insert(rules[key], rule)

      -- Closing key(s)
      for _, closing_key in ipairs(M.closing_keys_from_rule(key, rule)) do
        rules[closing_key] = rules[closing_key] or {}
        table.insert(rules[closing_key], rule)
      end
    end

    if rules[key] ~= nil then
      -- Sort by priority
      table.sort(rules[key], function(a, b) return a.priority > b.priority end)
    end
  end

  return rules
end

--- @param key string
--- @param def blink.pairs.RuleDefinition | string
--- @return blink.pairs.Rule
function M.rule_from_def(key, def)
  if type(def) == 'string' then
    return {
      opening = key,
      closing = def,
      priority = #key + #def,
      when = function() return true end,
      enter = function() return true end,
      backspace = function() return true end,
      space = function() return true end,
    }
  end

  local when = function()
    if def.filetypes ~= nil and not vim.tbl_contains(def.filetypes, vim.bo.filetype) then return false end
    return def.when == nil or def.when()
  end

  local closing = #def == 1 and def[1] or def[2]
  local opening = #def == 2 and def[1] or key

  local priority = #closing + #opening + (def.when ~= nil and 4 or 0)

  return {
    priority = def.priority or priority,
    closing = closing,
    opening = opening,
    when = when,
    enter = as_func(def.enter, true),
    backspace = as_func(def.backspace, true),
    space = as_func(def.space, true),
  }
end

--- TODO: only works for single character keys for now
--- @param key string
--- @param rule blink.pairs.Rule
--- @return string[]
function M.closing_keys_from_rule(key, rule)
  if key == rule.closing:sub(1, 1) then return {} end
  return { rule.closing:sub(1, 1) }
end

--- Checks if the rule's conditions mark it as active
--- @param rule blink.pairs.Rule
--- @param mode? blink.pairs.Mode
--- @return boolean
function M.is_active(rule, mode) return rule.when() and (mode == nil or rule[mode]()) end

--- @param rules blink.pairs.Rule[]
--- @param mode? 'enter' | 'backspace' | 'space'
--- @return blink.pairs.Rule?
function M.get_active(rules, mode)
  for _, rule in ipairs(rules) do
    if M.is_active(rule, mode) then return rule end
  end
end

--- @param rules blink.pairs.Rule[]
--- @param mode? 'enter' | 'backspace' | 'space'
--- @return blink.pairs.Rule[]
function M.get_all_active(rules, mode)
  return vim.tbl_filter(function(rule) return M.is_active(rule, mode) end, rules)
end

--- @param rules_by_key blink.pairs.RulesByKey
--- @return blink.pairs.Rule[] rules Sorted by priority
function M.get_all(rules_by_key)
  local all_rules = {}
  for _, rules in pairs(rules_by_key) do
    vim.list_extend(all_rules, rules)
  end
  table.sort(all_rules, function(a, b) return a.priority > b.priority end)
  return all_rules
end

--- Looks on either side of the cursor for existing pairs
--- @param rules blink.pairs.Rule[] Must be sorted by priority
--- @param mode? blink.pairs.Mode
--- @return blink.pairs.Rule? rule Rule surrounding the cursor
--- @return boolean? surrounding_space Whether there's a single space on either side of the cursor
function M.get_surrounding(rules, mode)
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()

  local before_cursor = line:sub(1, cursor[2])
  local after_cursor = line:sub(cursor[2] + 1)

  local has_surrounding_space = before_cursor:sub(-1) == ' ' and after_cursor:sub(1, 1) == ' '

  for _, rule in ipairs(rules) do
    if M.is_active(rule, mode) then
      -- Special case for backspace and enter where we ignore surrounding spaces, and return whether they're there
      if (mode == 'backspace' or mode == 'enter') and has_surrounding_space then
        if
          rule.opening == before_cursor:sub(-#rule.opening - 1, -2)
          and rule.closing == after_cursor:sub(2, #rule.closing + 1)
        then
          return rule, true
        end
      end

      if rule.opening == before_cursor:sub(-#rule.opening) and rule.closing == after_cursor:sub(1, #rule.closing) then
        return rule, false
      end
    end
  end
end

return M
