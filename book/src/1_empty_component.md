# Chapter 1: An Empty Component

As mentioned in the introduction, the simplest Asteracea component is `E()()[]`.

In context, and written more like what you'd see in the wild:

```rust asteracea=Empty::new()
asteracea::component! {
  pub Empty()()

  []
}
```

(All Asteracea component examples are followed by their output as rendered by [`lignin-html`], but in this case it's an empty string.)

[`lignin-html`]: https://github.com/Tamschi/lignin-html