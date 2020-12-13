# Constructor Block

TODO: Considering `with {…} <…>` expressions will exist to write procedural `render` code, it's a better idea to remove `do for 'RENDER {…}` procedures and to replace `do for 'NEW {…}` procedures with a single optional `do {…}`.

TODO (cont.): The reason for not placing this block after the constructor arguments (without keyword) is that this would create a lot of separation between constructor and render arguments, which should both be visible at a glance when peeking at a component's source code.
