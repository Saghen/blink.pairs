local utils = require('blink.pairs.utils')
local rule_lib = require('blink.pairs.rule')

local mappings = {}

--- @param rule_definitions blink.pairs.RuleDefinitions
function mappings.register(rule_definitions)
  local rules_by_key = rule_lib.parse(rule_definitions)

  local map = function(lhs, rhs) vim.keymap.set('i', lhs, rhs, { silent = true, noremap = true, expr = true }) end

  for key, rules in pairs(rules_by_key) do
    if #rules > 0 then map(key, mappings.on_key(key, rules)) end
  end

  local all_rules = rule_lib.get_all(rules_by_key)
  map('<BS>', mappings.backspace(all_rules))
  map('<CR>', mappings.enter(all_rules))
  map('<Space>', mappings.space(all_rules))
end

function mappings.on_key(key, rules)
  return function()
    local active_rules = rule_lib.get_all_active(rules)

    for _, rule in ipairs(active_rules) do
      -- TODO: set lazyredraw to prevent flickering

      if rule.opening == rule.closing then return mappings.open_or_close_pair(key, rule) end

      if #rule.opening == 1 then
        if rule.opening == key then return mappings.open_pair(key, rule) end
        return mappings.close_pair(rule)
      end

      -- Multiple characters

      local index_of_key = rule.opening:find(key)
      assert(index_of_key ~= nil, 'Key not found in rule (temporary limitation, contributions welcome!)')
      index_of_key = index_of_key - 1

      local opening_prefix = rule.opening:sub(1, index_of_key)

      -- I.e. user types '"' for line 'r#|', we expand to 'r#""#'
      -- or the pair is "'''", in which case the index_of_key is 0 because there's no relevant prefix
      if index_of_key == 0 or utils.is_before_cursor(opening_prefix) then
        return mappings.open_pair(key, rule, index_of_key + 1)
      end

      --- I.e. for line 'r#"', user types '"' to close the pair
      if utils.is_before_cursor(rule.opening) then return mappings.close_pair(rule) end
    end

    -- No applicable rule found
    return key
  end
end

--- @param amount number
--- @return string keycodes Characters to feed to neovim to move the cursor forward or backward
function mappings.shift_keycode(amount)
  if amount > 0 then return string.rep('<C-g>u<Right>', amount) end
  return string.rep('<C-g>u<Left>', -amount)
end

--- @param key string
--- @param rule blink.pairs.Rule
--- @param offset? number
function mappings.open_pair(key, rule, offset)
  -- \| -> \(|
  if mappings.is_escaped() then return key end
  -- | -> (|)
  return rule.opening:sub(offset or 0) .. rule.closing .. mappings.shift_keycode(-#rule.closing)
end

--- @param rule blink.pairs.Rule
function mappings.close_pair(rule)
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()
  local next_char = line:sub(cursor[2] + 1, cursor[2] + 1)
  local next_next_char = line:sub(cursor[2] + 2, cursor[2] + 2)

  -- |) -> )|
  if next_char == rule.closing:sub(1, 1) then return mappings.shift_keycode(#rule.closing) end
  -- | ) ->  )|
  if next_char == ' ' and next_next_char == rule.closing then return mappings.shift_keycode(-2) end

  return rule.closing
end

--- @param key string
--- @param rule blink.pairs.Rule
function mappings.open_or_close_pair(key, rule)
  -- \| -> \"|
  if mappings.is_escaped() then return key end

  local pair = rule.opening
  assert(pair == rule.closing, 'Opening and closing must be the same')

  -- |' -> '|
  if utils.is_after_cursor(pair) then return mappings.close_pair(rule) end

  -- \| -> \'|
  if mappings.is_escaped() then return key end

  -- Multiple character open
  -- '|' -> '''|'''
  if #rule.opening > 1 then
    local start_overlap = utils.find_overlap(utils.text_before_cursor(), rule.opening)
    local end_overlap = utils.find_overlap(utils.text_after_cursor(), rule.closing)
    local opening = rule.opening:sub(start_overlap + 1)
    local closing = rule.closing:sub(1, #rule.closing - end_overlap)

    return opening .. closing .. mappings.shift_keycode(-#closing)
  end

  -- | -> '|'
  return mappings.open_pair(key, rule)
end

--- @param rules blink.pairs.Rule[]
function mappings.backspace(rules)
  return function()
    local rule, surrounding_space = rule_lib.get_surrounding(rules, 'backspace')
    if rule == nil then return '<BS>' end

    -- ( | ) -> (|)
    -- TODO: disable in strings
    if surrounding_space then return '<Del><BS>' end

    -- (|) -> |
    return mappings.shift_keycode(#rule.closing) .. string.rep('<BS>', #rule.opening + #rule.closing)
  end
end

--- @param rules blink.pairs.Rule[]
function mappings.enter(rules)
  return function()
    local rule, surrounding_space = rule_lib.get_surrounding(rules, 'enter')
    if rule == nil then return '<CR>' end

    if surrounding_space then return mappings.shift_keycode(1) .. '<BS><BS>' .. '<CR><C-o>O' end

    -- (|) ->
    -- (
    --   |
    -- )
    return '<CR><C-o>O'
  end
end

--- @param rules blink.pairs.Rule[]
function mappings.space(rules)
  return function()
    local rule = rule_lib.get_surrounding(rules, 'space')
    if rule == nil then return '<Space>' end

    -- "(|)" -> "( | )"
    -- TODO: disable in strings
    return '<Space><Space>' .. mappings.shift_keycode(-1)
  end
end

function mappings.is_escaped()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()

  return line:sub(cursor[2], cursor[2]) == '\\'
end

return mappings
