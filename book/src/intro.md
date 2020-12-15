# Introduction

Welcome to the Asteracea guide book!

> **This book is a work in progress!**
>
> Until the first minor version stabilisation, there likely will only be this `develop`-branch version published online as rendered version, which usually won't mach a version of the crate published to crates.io. (Respective versions are tagged and can be rendered offline using (from the repository's root directory) `cd book` and `cargo run`.)
>
> In addition to the missing chapters, URLs are subject to change, links have have not been filled in and code blocks without highlighting or rendered HTML output may show unimplemented features.

## Audience

While using Asteracea effectively requires knowledge of Rust, this book is written also with non-Rustaceans in mind.

If you have experience with more traditional front-end frameworks like Angular or React, quite a few of the presented concepts should be familiar to you. I will also try to highlight these connections where they appear. By the end of the book, you should be able to read Asteracea's component templates and make modifications to them.

If you are already familiar with Rust, you can use the samples from [Chapter&nbsp;6: Integrating Asteracea] to create a complete web site or application including static site generation, server-side rendering and/or a (primary or optional) client-side approach.

[Chapter&nbsp;6: Integrating Asteracea]: TK

## Background

When I started using Rust privately in 2019, I had worked as a consultant on multiple web projects, mainly as front-end developer using TypeScript, Angular and React. I had grown increasingly frustrated with the failure classes of this weakly typed ecosystem: Aside from (rare but in those cases often destructive) run-ins with outdated or wrong type definitions for external packages, it was too easy to accidentally turn off type checking. It was often easy to forget to handle certain failure cases. React was quick to prototype in, but would often spiral in complexity and unchecked definitions on larger projects. Angular applications were comparatively robust but required manual subscription management to prevent memory leaks and required a significant amount of boilerplate that couldn't be abstracted away due to compiler limitations.

Meanwhile on the server side, Spring Boot was resource-hungry as a microservice platform, requiring powerful development systems to run a local test instance of the platform even without any data added to it. Using the documentation was also frustrating to me, since it was difficult to look up the various implicit behaviours. I wouldn't be able to work efficiently with such a system on my slower home computer that also needed to handle a considerable amount of browser tabs at the same time. To top it off, DTOs couldn't be easily shared through the various layers of the application.

I originally got into Rust to have another go at game development. This didn't go well at the time due to lack of high-level frameworks I could prototype something in quickly, but I liked the language and ended up writing several smaller utility programs. Then I had to switch Android ROMs to still get updates and lost the data stored in the finance tracker app I was using. (Backups were only available by uploading my data to the manufacturer's servers, which I decided against.) I took this as an opportunity to write my own tracker, to be hosted on a Pi Zero W so I could make entries from my phone. In part to learn about technologies I had seen but not used myself at work, I decided to use a network of Docker containers, with Postgres for storage and Nginx to serve static files and act as reverse proxy.

While this tracker project is currently stalled, with help from friends I still managed to create a successful prototype: With [Diesel], [Serde] and by [targeting WebAssembly], I could reuse a single DTO definition all the way from Diesel to the app's browser frontend. Resource usage was tiny, requiring only about 15MB of private RAM and less than 0.5% CPU for the entire idling prototype server! I was also looking forward to drop JSON from my stack when MsgPack and CBOR inspection was added to Firefox.

[Diesel]: https://diesel.rs/
[Serde ]: https://serde.rs/
[targeting WebAssembly]: https://www.rust-lang.org/what/wasm

However, here is where I hit a snag: I was used to relatively mature web frameworks that make it easy to write reusable components and test them in isolation via dependency injection. I was also looking for CSS scoping and to ideally *never* touch JavaScript myself (ideally skipping its build ecosystem entirely). I used version `0.1.0` of [`dodrio`] for a while, but as stated on its project page, it's not intended as complete GUI solution. [Iced] wasn't a good fit due to being more high-level than what I was going for. [`typed-html`] seemed close to what React does, but I was looking for more stateful component tooling. (`dodrio` inspired Asteracea's use of a bump allocator.)

[`dodrio`]: https://lib.rs/crates/dodrio
[Iced]: https://github.com/hecrj/iced
[`typed-html`]: https://github.com/bodil/typed-html

([Afterglow] did not exist at that point. You will probably want to look at it as an alternative before deciding what to go with. Its design goals seem different from Asteracea's, at a glance.)

[Afterglow]: https://github.com/extraymond/afterglow

I decided to write my own solution to this problem, which is where things started to escalate.

## Asteracea's Design Goals

Asteracea is, as of October 2020, still early in development and subject to change. However, there are a few main goals I want to enable with this framework that can be put into writing already:

- Low boilerplate:

  Web components have a certain shape that's shared between each of them. Creating a new component shouldn't require a large amount of text to get started, so that the focus is on what the individual component does.

  A completely empty component, minus the outer macro call, can be written as concisely as [`E()()[]`](./static_components/empty_component.md). This generates a (zero-size) model, a (practically empty) constructor and a render method that generates an empty element group - a VDOM node that results in no output. More complex components grow naturally from here.

  Formatting a value into the output can be as simple as [`!{value}`](). More on all this later.

- Straightforward macros:

  While Asteracea relies heavily on procedural macros, these macros aren't magic. By and large, Asteracea does a copy-and-paste source code transformation. (Some dynamic defaults exist. Criticism is welcome.)

  Code spans are preserved as much as possible, so if the input is valid to Asteracea but the output is invalid Rust, the relevant errors and warnings will appear in matching locations on the macro input.

- Inclusive runtime:

  At some point during development, Twitter made its new web interface mandatory for all users. As of October 2020, it is still quite heavy (topping `about:perfomance` in Firefox by a wide margin alongside YouTube), loads slowly, is next to impossible to style, occasionally glitchy and does not work whatsoever without JavaScript enabled.

  Asteracea can't take care of all of these things for you, but I'm proud to announce that serverside-rendering and static site generation are supported without specifically adjusting the application code. The clientside version of the app can then hydrate the existing DOM structure, whether seamlessly or with additional content not included in the static version.

  Asteracea has no signature pattern aside from capitalising element names (which saves on some runtime branching). Generated HTML and DOM are structured as if written by hand.

- Balancing safety, simplicity and generality:

  Asteracea inherits its safety and lifetime model from Rust, with the one part not validated by the compiler being the render loop, external to the core framework and main application code. This is due to interaction with the browser DOM at this point, though a different implementation using [`FinalizationRegistry`] may be possible there.

  [`FinalizationRegistry`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry

  The targeted application model is single-threaded, which means components and event handlers aren't required to be [`Send`] or [`Sync`].

  [`Send`]: https://doc.rust-lang.org/stable/std/marker/trait.Send.html
  [`Sync`]: https://doc.rust-lang.org/stable/std/marker/trait.Sync.html

  Event handlers are only required to be valid for one render cycle (though reusing closures is encouraged and done by the basic event handler syntax). Component instances are required to outlive event handlers, but their lifetime is otherwise unconstrained by default. In particular, you can usually drop component instances before their rendered VDOM iff they don't register event handlers.

  Any expression between curly brackets (`{}`) in the templates is plain old Rust: The code is always¹ pasted verbatim and you can use any and all Rust features in those locations.

  ¹ This is technically only effectively true: A small but limited find-and-replace transformation is applied to event handlers to enable using `self` within them. It should match expected Rust behaviour under all circumstances, though.

  Asteracea is named after the family of [Asteraceae], which contains very spectacular as well as very inconspicuous, but generally quite practical flowers. My hope is that this set of libraries will eventually be used for a similarly wide range of applications.

  [Asteraceae]: https://en.wikipedia.org/w/index.php?title=Asteraceae&oldid=982133740
