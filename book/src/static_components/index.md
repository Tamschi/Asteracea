# Chapter 1: Static Components

Asteracea, [unlike for example React], does not have multiple ways to define a component depending on whether you'd like to use instance state or not¹. Instead, due to low syntactic overhead and Rust's efficiency, `struct` components are generated throughout.

[unlike for example React]: https://reactjs.org/docs/components-and-props.html#function-and-class-components

Stateless `struct` components have zero runtime overhead compared to functions equivalent [`fragment!`] use. This, along with less boilerplate and for consistency, is why I generally recommend [`component!`] for all reusable GUI elements.

[`fragment!`]: TK
[`component!`]: TK

In this chapter, I will introduce the basics of generating various virtual DOM nodes in Asteracea, which can then be translated into (e.g.!) HTML or browser DOM elements.

¹ The distinction has weakened in React recently. Asteracea's approach to stateful components is [partially inspired] by React's Hooks in terms of UX, but is implemented very differently below the surface.

[partially inspired]: ./2_5_body_captures.md
