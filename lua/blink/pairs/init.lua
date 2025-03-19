-- TODO: injected languages for markdown
-- TODO: many many more language definitions

local pairs = {}

local function set_highlights()
  vim.api.nvim_set_hl(0, 'BlinkPairsOrange', { ctermfg = 15, fg = '#d65d0e', default = true })
  vim.api.nvim_set_hl(0, 'BlinkPairsPurple', { ctermfg = 13, fg = '#b16286', default = true })
  vim.api.nvim_set_hl(0, 'BlinkPairsBlue', { ctermfg = 12, fg = '#458588', default = true })
end

--- @param user_config blink.pairs.Config
function pairs.setup(user_config)
  set_highlights()

  local config = require('blink.pairs.config')
  config.merge_with(user_config)

  pairs.download_if_available(function(err)
    if err then error(err) end

    if config.mappings.enabled then require('blink.pairs.mappings').register(config.mappings.pairs) end
    if config.highlights.enabled then require('blink.pairs.highlighter').register(config.highlights) end
  end)
end

--- You may optionally use `blink.download` for prebuilt binaries with the included `Cross.toml`
--- and `.github/workflows/release.yaml`
function pairs.download_if_available(callback)
  local success, downloader = pcall(require, 'blink.download')
  if not success then return callback() end

  -- See https://github.com/Saghen/blink.download for more info
  local root_dir = vim.fn.resolve(debug.getinfo(1).source:match('@?(.*/)') .. '../../../')

  downloader.ensure_downloaded({
    -- omit this property to disable downloading
    -- i.e. https://github.com/Saghen/blink.pairs/releases/download/v0.1.0/x86_64-unknown-linux-gnu.so
    download_url = function(version, system_triple, extension)
      return 'https://github.com/saghen/blink.pairs/releases/download/' .. version .. '/' .. system_triple .. extension
    end,
    on_download = function()
      vim.notify('[blink.pairs] Downloading prebuilt binary...', vim.log.levels.INFO, { title = 'blink.pairs' })
    end,

    root_dir = root_dir,
    output_dir = '/target/release',
    binary_name = 'blink_pairs', -- excluding `lib` prefix
  }, callback)
end

return pairs
