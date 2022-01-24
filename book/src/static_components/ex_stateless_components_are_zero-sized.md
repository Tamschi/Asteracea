# EX: Stateless Components are Zero-Sized

<details>
<summary>This is an optional chapter.</summary>

> EX-chapters don't contain necessary information on how to use Asteracea.
>
> However, they may contain interesting information about performance characteristics or tricks you can use to make your app more maintainable.

</details>

Consider the following (grasping at [constructor parameter]()[ captures]() and [value formatting]() a bit ahead of time):

```rust asteracea=Container
use std::{fmt::Debug, mem::size_of};

asteracea::component! {
  MySize<T: Debug>(
    priv value: T,
  )()

  [
    !"I contain {:?}!"(self.value)
    " My size is " <b !(size_of::<Self>())> " bytes."
    !" I'm located at address {:p}."(self)
  ]
}

asteracea::component! {
  Container()()

  [
    <*MySize::<()> *value = {()}> "\n"
    <*MySize::<()> *value = {()}> "\n"
    <*MySize::<u8> *value = {1}> "\n"
    <*MySize::<usize> *value = {2}> "\n"
    "The container instance occupies " <b !(size_of::<Self>())> !" bytes at {:p}."(self)
  ]
}
```

The layout here is somewhat implementation-defined, but generally what you should see is that the `MySize::<()>` instances take up no space inside the `Container` instance and don't displace other children in memory.

This is because Asteracea components contain no hidden instance state, which means they are sized to content (and padding requirements), all the way down to zero. `()` is Rust's [unit](https://doc.rust-lang.org/stable/std/primitive.unit.html) type, the most convenient [zero sized type](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts). The same applies to components without instance fields, of course.

Zero-sizing is transitive and has many interesting implications, but for our purposes the most notable one is that stateless components are *almost*¹ function-like at runtime. It's for this reason that Asteracea doesn't provide a special "slim" component syntax.

> ¹ There is likely a small amount of overhead during instantiation due to the [dependency extraction](../dynamic_component/dependency_extraction.md) system.
> The compiler is in theory allowed to optimise it away, but this isn't guaranteed.
>
> If you have an idea how to make this process meaningfully conditional without changing Asteracea's macro syntax, I'd like to hear about it!
