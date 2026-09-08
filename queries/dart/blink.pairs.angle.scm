; List<int>, Map<K, List<V>>
(type_arguments) @pair.inside

(type_parameters) @pair.inside

; List<int> x, extends Foo<T>, Future<void> foo()
(type_identifier) @pair.inside_or_after

; class Foo<T>, mixin Foo<T>, extension Foo<T>
(class_definition
  name: (identifier) @pair.after)

(mixin_declaration
  (identifier) @pair.after)

(extension_declaration
  name: (identifier) @pair.after)

; void foo<T>()
(function_signature
  name: (identifier) @pair.after)

; incomplete function declaration, parsed as a variable
; void foo<
(initialized_identifier
  .
  (identifier) @pair.after
  .)

; List<int>(), foo<int>()
((identifier) @pair.after
  .
  (selector
    (argument_part)))

; incomplete declarations
; class Foo<, mixin Foo<
(ERROR
  [
    "class"
    "mixin"
    "extension"
  ]
  .
  (identifier) @pair.after)

; lone identifier at the start of a statement
; List<
((ERROR
  .
  (identifier) @pair.after
  .) @_error
  (#lua-match? @_error "^[%w_.]+$"))
