local function get_lib_extension()
  if jit.os:lower() == 'mac' or jit.os:lower() == 'osx' then return '.dylib' end
  if jit.os:lower() == 'windows' then return '.dll' end
  return '.so'
end

-- search for the lib in the /target/release directory with and without the lib prefix
-- since MSVC doesn't include the prefix
package.cpath = package.cpath
  .. ';'
  .. debug.getinfo(1).source:match('@?(.*/)')
  .. '../../../target/release/lib?'
  .. get_lib_extension()
  .. ';'
  .. debug.getinfo(1).source:match('@?(.*/)')
  .. '../../../target/release/?'
  .. get_lib_extension()

--- @class blink.pairs.Parser
--- @field parse_buffer fun(bufnr: number?, filetype: string, lines: string[], start_line: number?, old_end_line: number?, new_end_line: number?): boolean
--- @field get_line_matches fun(bufnr: number, line_number: number, token_type: number?): blink.pairs.Match[]
--- @field get_span_at fun(bufnr: number, row: number, col: number): string?
--- @field get_match_at fun(bufnr: number, row: number, col: number): blink.pairs.Match?
--- @field get_match_pair fun(bufnr: number, row: number, col: number): blink.pairs.MatchWithLine[]?
--- @field get_unmatched_opening_before fun(bufnr: number, opening: string, closing: string, row: number, col: number): blink.pairs.MatchWithLine?
--- @field get_unmatched_closing_after fun(bufnr: number, opening: string, closing: string, row: number, col: number): blink.pairs.MatchWithLine?

--- @class blink.pairs.Match
--- @field [1] string
--- @field [2] string?
--- @field span string?
--- @field col number
--- @field stack_height number?

--- @class blink.pairs.MatchWithLine : blink.pairs.Match
--- @field line number

--- @type blink.pairs.Parser
return require('blink_pairs')
