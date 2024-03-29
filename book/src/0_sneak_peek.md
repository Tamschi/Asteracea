# Chapter 0: Sneak Peek

Before I begin to explain in earnest, here is a relatively complex dynamic component using many of Asteracea's features, along with its resulting HTML representation:

```rust asteracea=CounterUser
use lignin::web::Event;
use std::cell::Cell;

fn schedule_render() { /* ... */ }

asteracea::component! {
  Counter(
    initial: i32,
    priv step: i32,
    pub enabled: bool = true,
  )(
    class?: &'bump str,
  ) -> !Sync

  let self.value = Cell::<i32>::new(initial);

  //

  <div
    .class? = {class}
    "The current value is: " !(self.value()) <br>

    <button
      .disabled? = {!self.enabled}
      "+" !(self.step)
      on bubble click = Self::on_click_plus
    >
  >
}

//

impl Counter {
  pub fn value(&self) -> i32 {
    self.value.get()
  }

  pub fn set_value(&self, value: i32) {
    self.value.set(value);
    schedule_render();
  }

  fn on_click_plus(&self, _: Event) {
    self.set_value(self.value() + self.step);
  }
}

asteracea::component! {
  CounterUser()() -> !Sync

  <"counter-user" "\n\t"
    <*Counter
      *initial = {0}
      *step = {1}
    > "\n"
  >
}
```

This guide assumes you have done some web development before, so some parts of the template should look familiar to you.

Others probably look pretty unfamilar, even with both a web development and Rust background. I removed some redundant grammar and had to invent new syntax for some features that don't appear as such in either ecosystem.

Overall, I like to call this an MVC lite approach: You can see the model, view and controller parts of the component, in this order, without them being separated into different files. I've marked the boundaries between parts with a Rust comment each (`//`).

This actually isn't mandatory - Asteracea is quite flexible and lets you mix them when appropriate - but it's a good way to clean up larger components that otherwise wouldn't fit on the screen well.

There's also syntax highlighting without extra tools! The version here in the book is simplified, but if you use [rust-analyzer], then it's really quite smart.

[rust-analyzer]: https://rust-analyzer.github.io/

The following chapters will teach you how to read and write these components, though becoming fluent may require a little bit of practice.
