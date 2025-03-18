local utils = {}

--- Checks if the text after the cursor is equal to the given text
--- @param text string
--- @param ignore_single_space? boolean
--- @return boolean
function utils.is_after_cursor(text, ignore_single_space)
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()

  if ignore_single_space then
    if line:sub(cursor[2] + 1, cursor[2] + 1) == ' ' then cursor[2] = cursor[2] + 1 end
  end

  return line:sub(cursor[2], cursor[2] + #text - 1) == text
end

--- Checks if the text before the cursor is equal to the given text
--- @param text string
--- @param ignore_single_space? boolean
--- @return boolean
function utils.is_before_cursor(text, ignore_single_space)
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()

  if ignore_single_space then
    if line:sub(cursor[2], cursor[2]) == ' ' then cursor[2] = cursor[2] - 1 end
  end

  return line:sub(cursor[2] - #text + 1, cursor[2]) == text
end

--- Gets the text that comes before the cursor
--- (i.e. "foo|bar" -> "foo")
--- @return string
function utils.text_after_cursor()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()
  return line:sub(cursor[2] + 1)
end

--- Gets the text that comes before the cursor
--- (i.e. "foo|bar" -> "bar")
--- @return string
function utils.text_before_cursor()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = vim.api.nvim_get_current_line()
  return line:sub(1, cursor[2])
end

--- Finds the maximum overlap between two strings (a and b)
--- from the end of "a" and beginning of "b"
--- @param a string
--- @param b string
--- @return number
function utils.find_overlap(a, b)
  for overlap = math.min(#a, #b), 1, -1 do
    if a:sub(-overlap) == b:sub(1, overlap) then return overlap end
  end
  return 0
end

--- TODO: Apparently there can be flicker in large files with treesitter enabled
--- Need to investigate this
--- @generic T
--- @param f fun(): T
--- @return T
function utils.with_lazyredraw(f)
  local lazyredraw = vim.o.lazyredraw
  vim.o.lazyredraw = true

  local success, result_or_err = pcall(f)

  vim.o.lazyredraw = lazyredraw

  if not success then error(result_or_err) end
  return result_or_err
end

--- Slices an array
--- @generic T
--- @param arr T[]
--- @param start number?
--- @param finish number?
--- @return T[]
function utils.slice(arr, start, finish)
  start = start or 1
  finish = finish or #arr
  local sliced = {}
  for i = start, finish do
    sliced[#sliced + 1] = arr[i]
  end
  return sliced
end

return utils
