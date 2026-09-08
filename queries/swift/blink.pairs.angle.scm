; Array<Int>, Dictionary<K, Array<V>>
(type_arguments) @pair.inside

(type_parameters) @pair.inside

; var x: Array<Int>, struct Foo<T>, x as Array<Int>, func foo() -> Array<Int>
(type_identifier) @pair.inside_or_after

; func foo<T>()
(function_declaration
  name: (simple_identifier) @pair.after)

; Array<Int>(), foo<Int>()
(call_expression
  .
  (simple_identifier) @pair.after)

; incomplete declarations
; func foo<, struct Foo<, class Foo: Bar<
(ERROR
  [
    "func"
    "struct"
    "class"
    "enum"
    "actor"
    "protocol"
    "extension"
    "typealias"
    ":"
  ]
  .
  (simple_identifier) @pair.after)
