; List<String>, Map<K, List<V>>
(type_arguments) @pair.inside

(type_parameters) @pair.inside

; List<String> x, new ArrayList<>(), extends Foo<T>, Map.Entry<K, V>
; excludes `a` in the partial `if (a b)`, which is parsed as a type
((type_identifier) @pair.inside_or_after
  (#not-has-parent? @pair.inside_or_after ERROR))

; class Foo<T>, interface Foo<T>, record Foo<T>()
(class_declaration
  name: (identifier) @pair.after)

(interface_declaration
  name: (identifier) @pair.after)

(record_declaration
  name: (identifier) @pair.after)

; public <T> void foo()
(modifiers) @pair.after

; incomplete declarations
; class Foo<, interface Foo<, private List<
(ERROR
  [
    "class"
    "interface"
    "record"
    (modifiers)
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
