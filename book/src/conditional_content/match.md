# `match {…} [ … ]`

Rust's `match` statements are available in Asteracea contexts, with slightly changed syntax:

```TODOrust TODOasteracea=Routed
// Not yet implemented and very much subject to change.
asteracea::component! {
  pub Routed()()

  match <*Router> [
    Router::INDEX | "" => { unreachable!() }
    "bio" => <*Bio>
    "projects" => <*Projects>
    "404" => <*Missing priv missing>
    other => <*{&self.missing .root={other}}>
  ]
}
```
