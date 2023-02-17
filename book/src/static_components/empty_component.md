# An Empty Component

As mentioned in the introduction, the simplest Asteracea component is `E()()[]`.

In context, and written more like what you'd see in the wild:

```rust asteracea=Empty
use asteracea::substrates::web;

asteracea::component! { web =>
  Empty()()

  []
}
```

(All Asteracea component examples are followed by their output as rendered by [`lignin-html`], but in this case it's an empty string.)

[`lignin-html`]: https://github.com/Tamschi/lignin-html

This component expands to the following Rust code, with `use` imports extracted by hand to improve readability:

```rust no_run noplayground
//TODO
```

As you can see, the `component!` macro created a `struct` type, with one constructor called `new` and one method called `render`, as well as a few helper types and functions that enable named arguments, and a [`Drop`](https://doc.rust-lang.org/stable/std/ops/trait.Drop.html) implementation. The output of `component!`, as far as you're supposed to touch it, **always** has this shape. No exceptions.

Identifiers containing `__Asteracea__` are considered internal and may change at any point in time. Please don't use them directly, even if technically accessible!

You may find small bits of similar useless syntax like those empty `{}` blocks in `new`. Some of these pieces of code nudge Rust into giving you a better error message or block off certain edge cases (usually inner attributes) that either would be confusing to read or haven't been properly evaluated yet, while others, like the empty `unsafe {}` in `drop` are slots where code is placed when generating more complex components, and which should be effectively removed by the compiler if empty. (If you notice such an empty construct that impacts runtime performance or Wasm assembly size, please file a bug report.)

## The breakdown

There are five distinct pieces of syntax that are translated into the output here: `pub`, `Empty`, `()`, `()` and `[]`.

### `pub` (visibility)

This is a plain [Rust visibility] and inserted just before the `struct` keyword in the macro output above, controlling where the component can be used directly. Leave it out to for current-module-only visibility.

[Rust visibility]: https://doc.rust-lang.org/stable/reference/visibility-and-privacy.html?highlight=pub#visibility-and-privacy

`new` and `render` are always declared `pub`; They inherit their visibility from the component structure.

### `Empty` (component name)

This identifier is inserted verbatim into the output as shown.

There aren't any requirements regarding *which* identifier to use, but I encourage you to avoid generic suffixes like "`…Component`".

Consider e.g. "`…ListItem`", "`…Button`" or, if nothing more specific applies, "`…Panel`" as more descriptive alternatives, or leave the suffix off entirefly if there's no confusion regarding which types are components and which are not.

### `()` (constructor argument list)

This is the first pair of parenthese in the input and also appears before the other in the output. As you can see, it is inserted verbatim after `new` here.

You can use any normal argument declaration here, with the exception of `self` parameters.

The constructor argument list also supports a shorthand to declare and assign fields on the component instance, but more on that [later].

### `()` (render argument list)

The second pair of parentheses is used to declare **additional** render arguments.

This one is never pasted verbatim into the resulting component, despite supporting only plain Rust argument declarations (with the exception of `self` parameters and, usually, `bump`).

Instead, its items are inserted at the end of `render`'s argument list above, after the implicit arguments `&self` and `bump: &'bump Bump`. You can access instance fields through `self` in the component body (more on that later) and `bump` is a [`Bump`] from [`bumpalo`], a bump allocation arena that makes the VDOM more efficient.

[`Bump`]: https://docs.rs/bumpalo/3/bumpalo/struct.Bump.html
[`bumpalo`]: https://github.com/fitzgen/bumpalo

**Do not place anything into `bump` that needs to be dropped!** Bump allocators are speedy, but this speed is bought by not running any logic before the memory is reused. Some workarounds for common use cases exist, but for the most part Asteracea handles this for you. See [`bumpalo`]'s documentation for more information.

[`bumpalo`]: https://github.com/fitzgen/bumpalo

### `[]` (body / empty Multi Node)

The location of `[]` in this example component is called the **body** of the component.

`[]` itself is an **empty Multi Node**, which expands to `Node::Multi(&*bump.alloc_with(|| []))`.

The contents of this node are placed in the bump allocation arena which, in this case, is effectively no operation. Location and length of this list are stored in the containing [`Node`], which here is returned directly from `render`.

It's legal to reuse [`Node`] instances in multiple places in the VDOM tree. You can also cache long-lived [`Node`]s and then refer to them across multiple render cycles, to avoid re-rendering part of the VDOM.

**Multi Nodes** are a VDOM concept that doesn't translate into DOM: Their contents are replicated without nesting in the surrounding DOM structure. You can use them, for example, to return multiple elements at the top level of a component.

Another use is to represent a variable number of elements, including none. The diffing algorithm in [`lignin-dom`] advances by a single VDOM sibling when processing a multi node. This means that you can avoid shifting any following sibling nodes, which can avoid expensively recreating their DOM representation or confusing the user by moving their selection to an unexpected place.

[`lignin-dom`]: https://github.com/Tamschi/lignin-dom
