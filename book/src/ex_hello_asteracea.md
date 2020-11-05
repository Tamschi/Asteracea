# Chapter 1: Hello Asteracea

```rust asteracea=HelloAsteracea::new()
asteracea::component! {
  HelloAsteracea()()
  <span "Hello Asteracea!">
}
```

```rust asteracea=Counter::new(0,1,"")
use asteracea::component;
use std::cell::Cell;

fn schedule_render() { /* ... */ }

component! {
  pub Counter(
    initial: i32,
    priv step: i32,
    /// This component's class attribute value.
    pub class: &'static str, // ⁵
  )()

  |value = Cell::<i32>::new(initial)|; // shorthand capture

  <div
    ."class" = {self.class} // ⁶
    "The current value is: " !{self.value()} <br> // Anything within curlies is plain Rust.

    <button
      "+" !{self.step} // shorthand bump_format call
      +"click" {
        self.value.set(self.value() + self.step);
        schedule_render();
      }
    >
  >
}

impl Counter {
  pub fn value(&self) -> i32 {
    self.value.get()
  }

  pub fn set_value(&self, value: i32) {
    self.value.set(value);
  }
}
```
