(type_arguments) @pair.inside

(type_identifier) @pair.inside_or_after

; incomplete function signature
; fn foo<
(ERROR
  .
  "fn") @pair.inside_or_after

"::" @pair.after
