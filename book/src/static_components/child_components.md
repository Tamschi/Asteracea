# Child Components

Asteracea components can be used inside other templates using asterisk syntax:

```rust asteracea=Outer
//TODO: Hide this initially.
use std::marker::PhantomData;

asteracea::component! {
  Inner()()

  "Inner body."
}

mod module {
  asteracea::component! {
    pub(crate) Module()()

    "Module body."
  }
}

asteracea::component! {
  Generic<T>(
    //TODO: Hide this initially and show an ellipsis comment.
    // Generic parameters must be used in an instance field.
    // We can pretend this is the case using a constructor parameter capture.
    // `PhantomData` is a type that provides fake storage semantics.
    priv _phantom: PhantomData<T> = PhantomData::default(),
  )()

  "Generic body."
}

asteracea::component! {
  Outer()()

  [
    <*Inner> "\n"
    <*module::Module> "\n"
    <*Generic::<()>> // Mind the turbofish! ::<> 🐟💨
  ]
}
```

Explicit closing is supported:

```rust asteracea=Outer
//TODO: Hide repetition.
use std::marker::PhantomData;

asteracea::component! {
  Inner()()

  "Inner body."
}

mod module {
  asteracea::component! {
    pub(crate) Module()()

    "Module body."
  }
}

asteracea::component! {
  Generic<T>(
    // Generic parameters must be used in an instance field.
    // We can pretend this is the case using a constructor parameter capture.
    // `PhantomData` is a type that provides fake storage semantics.
    priv _phantom: PhantomData<T> = PhantomData::default(),
  )()

  "Generic body."
}

asteracea::component! {
  Outer()()

  [
    <*Inner /Inner> "\n"
    <*module::Module /Module> "\n"
    <*Generic::<()> /Generic> // 🪣
  ]
}
```

<!--
I nearly put the FISHING POLE AND FISH emoji above, but that fest to cruel.
The fish is chilling in a bucket now and will be released into a nicer environment before long.
-->

Using a component multiple times results in distinct instances:

```rust asteracea=Outer
asteracea::component! {
  Inner()()
  "Inner body."
}

asteracea::component! {
  Outer()()
  [
    <*Inner>
    <*Inner>
  ]
}
```

## Child Component Instancing

> Note: Rust is good at erasing empty instances!
>
> If your reused component is stateless, please restate the component's type name instead of using instancing. This will keep your code clearer and less interdependent.
>
> For more information, see [The Rustonomicon on Zero Sized Types (ZSTs)](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts).

Instead of instantiating and storing a child component multiple times, you can instance it by giving it a name and referencing it elsewhere through a Rust block:

```rust asteracea=Outer
//TODO: Hide this initially.
asteracea::component! {
  Inner()()
  "Inner body."
}

asteracea::component! {
  Outer()()
  [
    <*Inner priv inner> // Alternatively: `pub` or `pub(…)`
    <*{self.inner_pinned()}>
  ]
}
```

The component's `.render(…)` method is called for each of these appearances, but `::new(…)` is called only once.

Component instancing is especially useful when rendering alternates, since the child instance is available everywhere in the parent component's body (regardless which `.render(…)` path is taken).
