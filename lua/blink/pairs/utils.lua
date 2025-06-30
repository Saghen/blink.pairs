local utils = {}

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

---@type boolean Have we passed UIEnter?
local _ui_entered = vim.v.vim_did_enter == 1 -- technically for VimEnter, but should be good enough for when we're lazy loaded
---@type function[] List of notifications.
local _notification_queue = {}

--- Fancy notification wrapper.
--- @param msg [string, string?][]
--- @param lvl? number
function utils.notify(msg, lvl)
  local header_hl = 'DiagnosticVirtualTextWarn'
  if lvl == vim.log.levels.ERROR then
    header_hl = 'DiagnosticVirtualTextError'
  elseif lvl == vim.log.levels.INFO then
    header_hl = 'DiagnosticVirtualTextInfo'
  end

  table.insert(msg, 1, { ' blink.pairs ', header_hl })
  table.insert(msg, 2, { ' ' })

  local echo_opts = { verbose = false }
  if lvl == vim.log.levels.ERROR and vim.fn.has('nvim-0.11') == 1 then echo_opts.err = true end
  if _ui_entered then
    vim.schedule(function() vim.api.nvim_echo(msg, true, echo_opts) end)
  else
    -- Queue notification for the UIEnter event.
    table.insert(_notification_queue, function() vim.api.nvim_echo(msg, true, echo_opts) end)
  end
end

vim.api.nvim_create_autocmd('UIEnter', {
  callback = function()
    _ui_entered = true

    for _, fn in ipairs(_notification_queue) do
      pcall(fn)
    end
  end,
})

return utils
