; Array<Int>, Map<K, Array<V>>
(type_params) @pair.inside

; var x: Array<Int>, function foo(): Array<Int>
(type
  type_name: (identifier) @pair.after)

; class Foo<T>, interface Foo<T>, typedef Foo<T>, function foo<T>()
(class_declaration
  [
    name: (identifier) @pair.after
    super_class_name: (identifier) @pair.after
  ])

(interface_declaration
  name: (identifier) @pair.after)

(typedef_declaration
  (identifier) @pair.after)

(function_declaration
  name: (identifier) @pair.after)

; new Array<Int>()
(call_expression
  "new"
  object: (identifier) @pair.after)

; incomplete declarations
; class Foo<, function foo<, var x: Array<
(ERROR
  [
    "class"
    "interface"
    "function"
    "typedef"
    "extends"
    ":"
  ]
  .
  (identifier) @pair.after)

; enum Foo<
((ERROR
  (identifier) @_keyword
  .
  (identifier) @pair.after)
  (#eq? @_keyword "enum"))
