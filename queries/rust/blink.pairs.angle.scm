; Vec<T>, impl<T> Foo<T>, T: Into<String>
(type_arguments) @pair.inside

(type_parameters) @pair.inside

(type_identifier) @pair.inside_or_after

; fn foo<T>()
(function_item
  name: (identifier) @pair.after)

(function_signature_item
  name: (identifier) @pair.after)

; incomplete function signature
; fn foo<
(ERROR
  .
  "fn") @pair.inside_or_after

; impl<T>, for<'a>
[
  "impl"
  "for"
] @pair.after

; turbofish
; foo::<T>()
"::" @pair.after
