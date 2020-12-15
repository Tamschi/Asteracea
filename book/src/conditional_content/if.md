# `if {…} <…>`

To conditionally render a node, you can use `if`-expressions whenever a [`Node<'bump>`]() is expected:

```TODOrust TODOasteracea=Conditioned
asteracea::compontent! {
  Conditional()(
    present: bool,
  )

  if {present}
    "I am here."
}

asteracea::component! {
  Conditioned()()

  [
    <*Conditional .present = {false}>
    <*Conditional .present = {true}>
  ]
}
```

Note the required curly braces (`{}`) around the condition and their absence on the body! This is reversed from plain Rust to show that the condition is a Rust expression while the body is not.

To render multiple elements conditionally, use a multi node:

```TODOrust TODOasteracea=Conditioned
asteracea::compontent! {
  Conditional()(
    present: bool,
  )

  [
    if {present} [ // <-- I recommend formatting this `[]` as you would format `{}` in Rust.
      "I am here"
      <span " and ">
    ]
    "I like this place."
}

asteracea::component! {
  Conditioned()()

  [
    <*Conditional .present = {false}> "\n"
    <*Conditional .present = {true}>
  ]
}
```

## Pattern-matching with `let`

is also available, though this means that Asteracea's `if`-`{condition}` is *not* automatically a Rust block. Use `{{ statements }}` if you really need one, though wrapping the `if` in a `with { … } <…>`-expression is likely a better idea in terms of code organisation.

```TODOrust TODOasteracea=Conditioned
asteracea::compontent! {
  Conditional()(
    content?: impl FnOnce(&Bump<'bump>) -> Node<'bump>,
  )

  [
    "["
    if {let Some(content) = content}
      { content(bump) }
    "]"
}

asteracea::component! {
  Conditioned()()

  [
    <*Conditional> "\n"
    <*Conditional .content = {|bump: &Bump<'bump>| asteracea::fragment! { // TODO:Use `.content => <…>` once that works.
      <div "Content!">
    }}>
  ]
}
```

> **Implicit `else`**
>
> If an `if`-expression's condition is not met, an empty [`Node::Multi(…)`]() (`[]`) is rendered by default.

<!-- intentionally split -->

> **A note for React users**
>
> Unlike React Hooks, Asteracea's captures (including `<*ChildComponent>`s) are generally fine to use in conditional `if`-branches, even if which branch is taken changes during the component's lifetime.
>
> The tradeoff for this is that their initialisers always run during component instantiation and that fields are created for any declared captures.
