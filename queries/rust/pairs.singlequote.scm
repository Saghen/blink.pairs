[
  (parameters)
  (type_arguments)
  (type_parameters)
  (reference_type)
] @nopair.inside

[
  (type_identifier)
  (reference_type)
] @nopair.inside_or_after

; incomplete struct definition
; struct Foo<
(ERROR
  .
  "struct") @nopair.inside_or_after

; incomplete function signature
; fn foo<
(ERROR
  .
  "fn") @nopair.inside_or_after

; emtpy function return type
; fn foo() -> {}
"->" @nopair.after
