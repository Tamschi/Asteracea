# Chapter 1: Hello Asteracea

```rust asteracea=HelloAsteracea
use asteracea::substrates::web;

asteracea::component! { web =>
  HelloAsteracea()()
  <span "Hello Asteracea!">
}
```

```rust asteracea=Counter asteracea::new=.initial(0).step(1) asteracea::render=.class("counter-class")
use asteracea::substrates::web;
use std::cell::Cell;

fn schedule_render() { /* ... */ }

asteracea::component! { web =>
  pub Counter(
    initial: i32,
    priv step: i32,
  )(
    /// This component's class attribute value.
    class?: &'bump str,
  )

  let self.value = Cell::<i32>::new(initial); // shorthand capture

  <div
    // Attribute usage is validated statically.
    // (Write its name as `str` literal to sidestep that.)
    // Anything within curlies is plain Rust.
    .class? = {class}

    // Three content nodes in this line,
    // with a shorthand bump_format call in the middle.
    "The current value is: " !(self.value()) <br>

    <button
      "+" !(self.step)

      // Correct event usage is validated statically.
      // (Write its name as `str` literal to sidestep that.)
      on bubble click = fn (self, _) { self.set_value(self.value() + self.step); } // Inline handler.
    >
  >
}

impl Counter {
  pub fn value(&self) -> i32 {
    self.value.get()
  }

  pub fn set_value(&self, value: i32) {
    self.value.set(value);
    schedule_render();
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
