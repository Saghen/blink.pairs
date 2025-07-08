# Blink Pairs (blink.pairs)

Rainbow highlighting and auto-pairs for Neovim. Uses a custom parser internally which takes ~2ms to parse a 400k character file, and ~0.15ms for subsequent incremental updates. See [the roadmap](https://github.com/Saghen/blink.pairs/issues/9) for the current status, contributions welcome!

## Behavior

The behavior was inspired by [lexima.vim](https://github.com/cohama/lexima.vim) and [nvim-autopairs](https://github.com/windwp/nvim-autopairs)

| Before   | Input   | After    |
|----------|---------|----------|
| `\|`       | `(`       | `(\|)`     |
| `\|)`      | `(`       | `(\|)`     |
| `\|`       | `"`       | `"\|"`     |
| `""\|`     | `"`       | `"""\|"""` |
| `''\|`     | `'`       | `'''\|'''` |
| `\\|`       | `[`       | `\[\|`     |
| `\\|`       | `"`       | `\"\|`     |
| `\\|`       | `'`       | `\'\|`     |
| `A`        | `'`       | `A'`       |
| `(\|)`     | `)`       | `()\|`     |
| `((\|)`     | `)`       | `(()\|)`     |
| `'\|'`     | `'`       | `''\|`     |
| `'''\|'''` | `'`       | `''''''\|` |
| `(\|)`     | `<BS>`    | `\|`       |
| `'\|'`     | `<BS>`    | `\|`       |
| `( \| )`   | `<BS>`    | `(\|)`     |
| `(\|)`     | `<Space>` | `( \| )`   |

## Installation

```lua
{
  'saghen/blink.pairs',
  version = '*', -- (recommended) only required with prebuilt binaries

  -- download prebuilt binaries from github releases
  dependencies = 'saghen/blink.download',
  -- OR build from source, requires nightly: https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust
  -- build = 'cargo build --release',
  -- If you use nix, you can build from source using latest nightly rust with:
  -- build = 'nix run .#build-plugin',

  --- @module 'blink.pairs'
  --- @type blink.pairs.Config
  opts = {
    mappings = {
      -- you can call require("blink.pairs.mappings").enable() and require("blink.pairs.mappings").disable() to enable/disable mappings at runtime
      enabled = true,
      -- you may also disable with `vim.g.pairs = false` (global) or `vim.b.pairs = false` (per-buffer)
      disabled_filetypes = {},
      -- see the defaults: https://github.com/Saghen/blink.pairs/blob/main/lua/blink/pairs/config/mappings.lua#L12
      pairs = {},
    },
    highlights = {
      enabled = true,
      groups = {
        'BlinkPairsOrange',
        'BlinkPairsPurple',
        'BlinkPairsBlue',
      },
      unmatched_group = 'BlinkPairsUnmatched',
      matchparen = {
        enabled = true,
        group = 'MatchParen',
      },
    },
    debug = false,
  }
}
```
