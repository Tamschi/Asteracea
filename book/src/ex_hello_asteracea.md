# Chapter 1: Hello Asteracea

```rust asteracea=HelloAsteracea
asteracea::component! {
  HelloAsteracea()()
  <span "Hello Asteracea!">
}
```

```rust asteracea=Counter asteracea::new=.initial(0).step(1) asteracea::render=.class("counter-class")
use asteracea::component;
use std::cell::Cell;

fn schedule_render() { /* ... */ }

component! {
  pub Counter(
    initial: i32,
    priv step: i32,
  )(
    /// This component's class attribute value.
    class?: &'bump str,
  )

  |value = Cell::<i32>::new(initial)|; // shorthand capture

  <div
    ."class"? = {class}
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

<!-- markdownlint-disable no-inline-html -->
<div class="subtlish">
<style>
.subtlish {
  height: 0px;
}
</style>
<br><br><br><br><br><br><br><br>

🌬️🍃🌄  
🏞️🐟🪣
</div>
<!-- markdownlint-enable no-inline-html -->
