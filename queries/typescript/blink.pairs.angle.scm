; Map<string, T>, foo<T>()
(type_arguments) @pair.inside

(type_parameters) @pair.inside

; let x: Map, class Foo, interface Foo, type Foo, extends Foo, as Foo
(type_identifier) @pair.inside_or_after

; React.FC
(nested_type_identifier) @pair.inside_or_after

; function foo<T>()
(function_declaration
  name: (identifier) @pair.after)

(generator_function_declaration
  name: (identifier) @pair.after)

(function_expression
  name: (identifier) @pair.after)

(function_signature
  name: (identifier) @pair.after)

(method_definition
  name: (property_identifier) @pair.after)

(method_signature
  name: (property_identifier) @pair.after)

(abstract_method_signature
  name: (property_identifier) @pair.after)

; incomplete declarations
; function foo<, class Foo<, interface Foo<, type Foo<
(ERROR
  [
    "function"
    "class"
    "interface"
    "type"
  ]
  .
  (identifier) @pair.after)

; class Foo extends Bar<T>
(extends_clause
  value: (identifier) @pair.after)

; useState<string>(), foo.bar<T>(), new Map<string, T>()
(call_expression
  function: [
    (identifier) @pair.after
    (member_expression
      property: (property_identifier) @pair.after)
  ])

(new_expression
  constructor: [
    (identifier) @pair.after
    (member_expression
      property: (property_identifier) @pair.after)
  ])
