# `with {…} <…>`

While you can in theory place nearly any Rust code inside `{}`-braces as part of Asteracea's grammar, this can be disorderly for more complex calculations or cumbersome where you want to reuse parts of a calculation.

Instead, you can use a `with {…} <…>`-expression to run a number of Rust statements procedurally:

```TODOrust TODOasteracea=WithExample
asteracea::component! {
  pub WithExample()()

  with {
    let tree_types = "oak";
    let leaves_state = "fallen";
  } <div
    .class = !"{} {}"{tree_types, leaves_state}
    !"The tree in the garden is an {}.\n"{ tree_type }
    !"The {tree}'s leaves are {leaves}."{ tree = tree_type, leaves = leaves_state }
  >
}
```

Bindings declared in the `with`-expression's are only in scope the one [`Node`]() creation, but with a multi node, you can use them for multiple elements:

```TODOrust TODOasteracea=WithExample
asteracea::component! {
  pub WithExample()()

  <div
    with {
      let tree_types = "oak";
      let leaves_state = "fallen";
    } [
      !"The tree in the garden is an {}.\n"{ tree_type }
      !"The {tree}'s leaves are {leaves}."{ tree = tree_type, leaves = leaves_state }
    ]
  >
}
```
