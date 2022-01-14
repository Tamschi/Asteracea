# `defer ⦃storage⦄ <…>`

An alternative method of breaking up recursive component initialisation is to defer it for the recursive part of the template until it is rendered.

As such, the recursive example from the [`box ⟦priv …⟦: ⟦struct⟧ … ⟦where …;⟧⟧⟧ <…>` chapter](./box.md) can be written as:

```rust asteracea=HalfADozen
asteracea::component! {
  Countdown()(
    i: usize,
  ) -> Sync // Syncness hint required due to recursion.

  [
    !{i}
    spread if {i > 0} [
      "\n"
      defer box <*Countdown .i = {i - 1}>
    ]
  ]
}

asteracea::component! {
  pub HalfADozen()() -> Sync?

  <*Countdown .i = {6}>
}
```

This has different runtime characteristics:

`spread if` doesn't drop branches that aren't active, and `defer` only ever runs initialisers once. This means that **state persists** and **heap allocations are cached**. Useful for (very) frequently updated content!

## Naming the field

As usual, the backing field can be named and then referred to:

```rust asteracea=Ruminating
asteracea::component! {
  Introspective()()

  [
    "Was I rendered before? " !{self.deferred_pinned().get().is_some()}
    defer priv deferred: struct Deferred []
  ]
}

asteracea::component! {
  pub Ruminating()() -> Sync?

  [
    <*Introspective priv introspective> "\n"
    <*{self.introspective_pinned()}>
  ]
}
```

The subexpression backing storage is wrapped in a [`Deferred`]() instance, which provides a [`.get()`]() method returning an `Option<Pin<&Deferred>>` depending on whether the subexpression was constructed yet.

You can also call [`.get_or_poison()`](), to evaluate the constructor if pending, which returns a `Result<Pin<&Deferred>, Escalation>`.
