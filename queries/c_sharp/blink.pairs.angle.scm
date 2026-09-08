; List<string>, Dictionary<K, List<V>>
(type_argument_list) @pair.inside

(type_parameter_list) @pair.inside

; List<string> x, public List<string> Foo(), new List<string>(), void Foo(List<string> x)
(variable_declaration
  type: (identifier) @pair.after)

(method_declaration
  returns: (identifier) @pair.after)

(object_creation_expression
  type: (identifier) @pair.after)

(parameter
  type: (identifier) @pair.after)

(cast_expression
  type: (identifier) @pair.after)

(typeof_expression
  type: (identifier) @pair.after)

(as_expression
  right: (identifier) @pair.after)

; class Foo<T>, interface Foo<T>, struct Foo<T>, record Foo<T>, delegate void Foo<T>()
(class_declaration
  name: (identifier) @pair.after)

(interface_declaration
  name: (identifier) @pair.after)

(struct_declaration
  name: (identifier) @pair.after)

(record_declaration
  name: (identifier) @pair.after)

(delegate_declaration
  name: (identifier) @pair.after)

; void Foo<T>()
(method_declaration
  name: (identifier) @pair.after)

; incomplete method declaration, parsed as a field
; void Foo<
(variable_declarator
  name: (identifier) @pair.after)

; class Foo : Bar<T>, where T : IFoo<T>
(base_list
  (identifier) @pair.after)

(type_parameter_constraint
  type: (identifier) @pair.after)

; Foo<int>(), foo.Bar<int>()
(invocation_expression
  function: [
    (identifier) @pair.after
    (member_access_expression
      name: (identifier) @pair.after)
  ])

; incomplete declarations
; private List<
(ERROR
  (modifier)
  .
  (identifier) @pair.after)

; lone identifier at the start of a statement
; List<
((ERROR
  .
  (identifier) @pair.after
  .) @_error
  (#lua-match? @_error "^[%w_.]+$"))
