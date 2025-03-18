--- @class (exact) blink.pairs.Config : blink.pairs.ConfigStrict, {}
--- @field mappings? blink.pairs.MappingsConfigPartial
--- @field highlights? blink.pairs.HighlightsConfigPartial
--- @field debug? boolean

--- @class (exact) blink.pairs.MappingsConfigPartial : blink.pairs.MappingsConfig
--- @field enabled? boolean
--- @field pairs? blink.pairs.RuleDefinitions

--- @class (exact) blink.pairs.RuleDefinitions
--- @field [string] string | blink.pairs.RuleDefinition | blink.pairs.RuleDefinition[]

--- @class (exact) blink.pairs.RuleDefinition
--- @field [1] string Closing character (e.g. { ')' }) or opening character if two characters are provided (e.g. {'(', ')'})
--- @field [2]? string Closing character (e.g. {'(', ')'})
--- @field priority? number
--- @field filetypes? string[]
--- @field when? fun(): boolean
--- @field enter? boolean | fun(): boolean
--- @field backspace? boolean | fun(): boolean
--- @field space? boolean | fun(): boolean

--- @class (exact) blink.pairs.HighlightsConfigPartial : blink.pairs.HighlightsConfig, {}
