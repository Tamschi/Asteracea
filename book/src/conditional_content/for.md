# `for`-loops

`for` loops in Asteracea components resemble those in plain Rust, but do produce output on each iteration:

```rust asteracea=Looping
asteracea::component! { substrate =>
  pub Looping()() -> Sync

  for word in "This is a test.".split(' ') {[
      <li
        !"{:?}"(word)
      > "\n"
  ]}
}
```

You can declare bindings and use child components within the loop's body as normal.

Their state is:

- initialised anew for each new element,
- "stuck" to the respective element in the input sequence when it is reordered and
- dropped if an element has disappeared.

Repeated elements each have their own state, but are considered interchangeable.
Each such group's state "list" is auto-edited only at the end.

> DOM elements are also reordered for sequence updates, but this isn't entirely reliable if you use keys longer than 32 bits.
>
> This is guaranteed to never cause an inconsistent GUI state (since any output and bindings will still be updated to be in the correct order when diffing), but in very rare cases could result in a selection or focus "jumping elsewhere".
>
> Such glitches are guaranteed to never happen while the sequence is stable, however.

The full explicit syntax for `for`-loops is as follows:

```rust asteracea=Looping
asteracea::component! { substrate =>
  pub Looping()() -> Sync

  for i: u8 keyed &*i => u8 in 0..255 {
    "."
  }
}
```

where

- `i` is the item pattern, used both for the loop body (by value) and the selector,
- `: ⦃T⦄` determines the type of items in the sequence,
- `keyed ⦃selector⦄` projects `&mut T` to `&Q` where `Q: Eq + ToOwned<Owned = K>`,
- `=> ⦃K⦄` where `K: ReprojectionKey` determines the type of state keys cached internally and
- `0..255` is an [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) to use as item source.

Each of these parts is optional, but currently it may be occasionally necessary to specify a type. **Specifying neither `T` nor `K` also means a loop will use somewhat less efficient dynamically typed stored keys that always incur a heap allocation when an item is added to the list.** Both most annotation requirements and relative inefficiency of unannotated loops is expected to disappear with [future Rust language improvements](https://github.com/rust-lang/rust/issues/63063).
