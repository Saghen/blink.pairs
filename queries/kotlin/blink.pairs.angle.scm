; List<String>, Map<K, List<V>>
(type_arguments) @pair.inside

(type_parameters) @pair.inside

; val x: List<String>, class Foo<T>, x as List<String>, fun foo(): List<String>
(type_identifier) @pair.inside_or_after

; fun <T> foo()
"fun" @pair.after

; listOf<String>(), foo.bar<String>()
(call_expression
  .
  [
    (simple_identifier) @pair.after
    (navigation_expression
      (navigation_suffix
        (simple_identifier) @pair.after))
  ])
