# `defer ⟦move⟧ ⟦as …⟧ <…>`

An alternative method of breaking up recursive component initialisation is to defer it for the recursive part of the template until it is rendered.

As such, the recursive example from the [`box ⟦as …⟧ <…>` chapter](./box_as.md) can be written as:

```TODOrust TODOasteracea=Countdown asteracea::render=.i(6)
asteracea::component! {
  Countdown()(
    i: usize,
  )

  [
    !{i}
    spread if {i > 0} [
      "\n"
      defer box <*Countdown .i = {i - 1}>
    ]
  ]
}
```

This has different runtime characteristics:

`spread if` doesn't drop branches that aren't active, and `defer` only ever runs initialisers once. This means that **state persists** and **heap allocations are cached**. Useful for (very) frequently updated content!
