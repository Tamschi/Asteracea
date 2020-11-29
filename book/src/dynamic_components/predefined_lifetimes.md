# Predefined Lifetimes

Asteracea implicitly defines certain user-accessible lifetimes:

- `'a` is available in the `::new(…)` and `.render(…)` contexts and represents a lower bound of the component's lifetime.

- `'bump` is the bump allocator lifetime, used when rendering the virtual DOM representation from Asteracea components and only available in the `.render(…)` context. This is mostly implicit, but you have to specify it in render arguments where references flow into the VDOM.

Overall, the following are true, if types and values represent their lifetimes:

- `Self` ≥ `self`
- `self` == `'a`
- `'a` ≥ `'bump`

TODO: More details, especially regarding event handlers. Remove `Self: 'static` constraint.