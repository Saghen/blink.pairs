--- @class (exact) blink.pairs.Config : blink.pairs.ConfigStrict, {}
--- @field mappings? blink.pairs.MappingsConfigPartial
--- @field highlights? blink.pairs.HighlightsConfigPartial
--- @field debug? boolean

--- @class (exact) blink.pairs.MappingsConfigPartial : blink.pairs.MappingsConfig
--- @field enabled? boolean
--- @field disabled_filetypes? string[]
--- @field pairs? blink.pairs.RuleDefinitions

--- @alias blink.pairs.RuleDefinitions table<string, string | blink.pairs.RuleDefinition | blink.pairs.RuleDefinition[]>

--- @class (exact) blink.pairs.RuleDefinition
--- @field [1] string Closing character (e.g. { ')' }) or opening character if two characters are provided (e.g. {'(', ')'})
--- @field [2]? string Closing character (e.g. {'(', ')'})
--- @field priority? number
--- @field cmdline? boolean
--- @field languages? string[]
--- @field when? fun(ctx: blink.pairs.Context): boolean
--- @field open? boolean | fun(ctx: blink.pairs.Context): boolean Whether to open the pair
--- @field close? boolean | fun(ctx: blink.pairs.Context): boolean Whether to close the pair
--- @field open_or_close? boolean | fun(ctx: blink.pairs.Context): boolean Whether to open or close the pair, used in-place of `open` and `close` when the open and close are the same (such as for '' or "")
--- @field enter? boolean | fun(ctx: blink.pairs.Context): boolean
--- @field backspace? boolean | fun(ctx: blink.pairs.Context): boolean
--- @field space? boolean | fun(ctx: blink.pairs.Context): boolean

--- @class (exact) blink.pairs.HighlightsConfigPartial : blink.pairs.HighlightsConfig, {}
--- @field matchparen? blink.pairs.MatchparenConfigPartial

--- @class (exact) blink.pairs.MatchparenConfigPartial : blink.pairs.MatchparenConfig, {}
