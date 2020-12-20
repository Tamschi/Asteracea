# `box ⟦as …⟧ <…>`

A `box ⟦as …⟧ <…>` expression moves its parameter's backing storage into a new named heap object.

In practical terms:

- Any captures are first placed in a [`Box<…>`]() before being assigned to a ⟦named⟧ field inside the current component instance.

  This means that, for example, if you write `box as boxed <*Component as child>`, you'll need to access it as `self.boxed.child`.

  This **introduces some level of runtime indirection**, also when rendering components.

- **Uninitialised boxed expressions take up very little space.**

- **Recursion becomes possible.**

## Component recursion

Infinite recursive (storage) inlining isn't possible (except *theoretically* for zero-sized-types, but Rust makes no distinction here).

This means the following requires boxing:

```TODOrust TODOasteracea=Countdown asteracea::render=.i(6)
asteracea::component! {
  Countdown()(
    i: usize,
  )

  [
    !{i}
    dyn if {i > 0} [
      "\n"
      box <*Countdown .i = {i - 1}>
    ]
  ]
}
```

> **Note:**
>
> It's decidedly better to implement the above with a loop!  
> If you have a better example to demonstrate recursion with, please [let me know]()!

Note the use of `dyn if` to prevent infinite eager initialisation.

You can alternatively combine `spread if` with `lazy ⟦move⟧` to avoid throwing away heap allocations once they exist. This is better in cases where the recursion depth changes quickly or 

## Memory savings

The container component size reduction isn't very useful in most cases, since Asteracea initialises child components eagerly, but can be used to great effect with **`dyn` branching if certain arms require especially large storage**.

<!-- TODO: Check if it's possible to let Clippy warn about that. -->

Consider the following:

```rust asteracea=Holder
use std::mem::{MaybeUninit, size_of};

asteracea::component! {
  Heavy()()

  |large: [u8; 1_000] = {[0; 1_000]}|; // 1 KB
  "Hello!"
}

asteracea::component! {
  Holder()(
    show: bool = false,
  )

  [
    "Holder size: " !{size_of::<Self>()} " bytes"
    spread if {show} //TODO: Replace `spread` with `dyn`!
      <*Heavy>
  ]
}
```

As you can see, `Holder` requires 1KB of space even though `Heavy` is never used.

You can avoid this as follows:

```rust asteracea=Holder
use std::mem::{MaybeUninit, size_of};

asteracea::component! {
  Heavy()()

  |large: [u8; 1_000] = {[0; 1_000]}|; // 1 KB
  "Hello!"
}

asteracea::component! {
  Holder()(
    show: bool = false,
  )

  [
    "Holder size: " !{size_of::<Self>()} " bytes"
    spread if {show} //TODO: Replace `spread` with `dyn`!
      /*box*/ <*Heavy>
  ]
}
```

As `dyn if` won't initialise its branches unless necessary, no heap allocation happens in this case either.

<!-- TODO: Is there any way to demo this? -->
