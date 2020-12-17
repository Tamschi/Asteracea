# `with { …; } <…>`

While you can in theory place nearly any Rust code inside `{}`-braces as part of Asteracea's grammar, this can be disorderly for more complex calculations or cumbersome where you want to reuse parts of a calculation.

Instead, you can use a `with { …; } <…>`-expression to run a number of Rust statements procedurally:

```rust asteracea=WithExample
asteracea::component! {
  pub WithExample()()

  with {
    let tree_type = "oak";
    let leaves_state = "fallen";
  } <div
    //TODO: ."class" = !"{} {}"{tree_type, leaves_state}
    !"The tree in the garden is an {}.\n"{ tree_type }
    !"The {}'s leaves are {}."{tree_type, leaves_state} //TODO: Support named formatting parameters.
  >
}
```

`with`-expressions can be used anywhere an [element expression](*) is expected.

Bindings declared in the `with`-expression's are only in scope for the embedded [element expression](*), but with a multi node, you can use them for multiple elements:

```rust asteracea=WithExample
asteracea::component! {
  pub WithExample()()

  <div
    with {
      let tree_type = "oak";
      let leaves_state = "fallen";
    } [
      !"The tree in the garden is an {}.\n"{ tree_type }
      !"The {}'s leaves are {}."{tree_type, leaves_state} //TODO: Support named formatting parameters.
    ]
  >
}
```
