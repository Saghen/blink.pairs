-- delimiters.lua - Utility functions for working with delimiters

local M = {}

--- Get active delimiters from configured rules
--- @return table|nil Delimiter information or nil if unavailable
function M.get_active_delimiters()
  local global_config = require('blink.pairs.config')

  if
    not (global_config and global_config.mappings and global_config.mappings.enabled and global_config.mappings.pairs)
  then
    return nil
  end

  local rule_lib = require('blink.pairs.rule')
  local rules_by_key = rule_lib.parse(global_config.mappings.pairs)

  local all_rules = rule_lib.get_all(rules_by_key)
  local active_rules = rule_lib.get_all_active(all_rules)

  if not active_rules or #active_rules == 0 then return nil end

  return M.extract_from_rules(active_rules)
end

--- Get active rules from configuration
--- @return table|nil Array of active rules or nil if unavailable
function M.get_active_rules()
  local global_config = require('blink.pairs.config')

  if
    not (global_config and global_config.mappings and global_config.mappings.enabled and global_config.mappings.pairs)
  then
    return nil
  end

  local rule_lib = require('blink.pairs.rule')
  local rules_by_key = rule_lib.parse(global_config.mappings.pairs)
  local all_rules = rule_lib.get_all(rules_by_key)
  return rule_lib.get_all_active(all_rules)
end

--- Extract delimiter information from active rules
--- @param rules table Array of rule objects
--- @return table Structured delimiter information
function M.extract_from_rules(rules)
  local opening = {}
  local closing = {}
  local pairs = {}

  local result = {
    opening = opening,
    closing = closing,
    pairs = pairs,
  }

  for i = 1, #rules do
    local rule = rules[i]
    local rule_opening = rule.opening
    local rule_closing = rule.closing

    if not opening[rule_opening] then
      opening[rule_opening] = true
      opening[rule_opening:sub(1, 1)] = true
    end

    if not closing[rule_closing] then
      closing[rule_closing] = true
      closing[rule_closing:sub(1, 1)] = true
    end

    pairs[rule_opening] = rule_closing
    pairs[rule_closing] = rule_opening
  end

  return result
end

--- Check if a string is a configured delimiter
--- @param text string Text to check
--- @param delimiters table Delimiter information
--- @param delimiter_type? string Optional type ('opening' or 'closing')
--- @return boolean Whether the text is a configured delimiter
function M.is_delimiter(text, delimiters, delimiter_type)
  if not (text and delimiters) then return false end

  if delimiter_type then
    local type_delimiters = delimiters[delimiter_type]
    return type_delimiters and type_delimiters[text] or false
  else
    return (delimiters.opening and delimiters.opening[text])
      or (delimiters.closing and delimiters.closing[text])
      or false
  end
end

--- Get matching delimiter for a given one
--- @param text string Delimiter text
--- @param delimiters table Delimiter information
--- @return string|nil Matching delimiter or nil if not found
function M.get_matching(text, delimiters)
  if not (text and delimiters and delimiters.pairs) then return nil end

  return delimiters.pairs[text]
end

--- Find rule for a given opening or closing delimiter
--- @param text string Opening or closing delimiter text
--- @return table|nil Rule object or nil if not found
function M.find_rule_for_delimiter(text)
  if not text then return nil end

  local active_rules = M.get_active_rules()
  if not active_rules then return nil end

  for i = 1, #active_rules do
    local rule = active_rules[i]
    if rule.opening == text or rule.closing == text then return rule end
  end

  return nil
end

return M
