# Chapter 1: Hello Asteracea

```rust,asteracea=HelloAsteracea
asteracea::component! {
  HelloAsteracea()()
  <span "Hello Asteracea!">
}
```

```rust,asteracea=SlightlyComplex
asteracea::component! {
  SlightlyComplex()()
  [
    "Hello Asteracea!"
    <div
      <button "Click me!">
      <div "This is a slightly more complex example.">
    >
  ]
}
```

```rust
asteracea::component! {
  HelloAsteracea()()
  "Hello Asteracea!"
}
```
