//! Component traits, plus modules that explain features.\
//! [`components!`](`crate::components!`) implements and uses these for you.

use std::{future::Future, pin::Pin};

use pinned_init::PinInit;
use rhizome::{Repository, Waypoint};

use crate::{substrate::Substrate, SizeHint};

#[doc(inline)]
/// `let pattern: Type = @mode!(…);` statements.
///
/// These statements persist values in some shape or form
/// (depending on the chosen `mode` macro below) and bind it in
/// [`Init::init`]/[`InitAsync::init_async`] and/or [`Render::render`].
///
/// ```rust
/// # use std::marker::PhantomPinned;
/// #
/// # struct Pinned {
/// # 	_phantom: PhantomPinned
/// # }
/// #
/// # impl Pinned {
/// # 	fn new() -> Self {
/// # 		Self { _phantom: PhantomPinned }
/// # 	}
/// #
/// # 	fn when_pinned(self: Pin<&Self>) {
/// # 		let _ = self;
/// # 	}
/// #
/// # 	fn when_pinned_mut(self: Pin<&mut Self>) {
/// # 	}
/// # }
/// #
/// asteracea::components! {
/// 	#![substrate = Sub]
///
/// 	Test => {
/// 		let a = @pin!(Pinned::new());
/// 		let mut b = @pin!(Pinned::new());
///
/// 		({
/// 			// In `init`.
/// 			a.when_pinned();
/// 			b.when_pinned();
/// 			b.as_mut().when_pinned_mut();
/// 		})
/// 		[{
/// 			// In `render`.
/// 			a.when_pinned();
/// 			b.when_pinned();
/// 			b.as_mut().when_pinned_mut();
/// 		}]
/// 	}
/// }
/// ```
///
/// To create plain bindings and variables in `init` or `render`,
/// instead use `(let) …;` or `[let] …;` statements, respectively,
/// with usual Rust syntax.
pub use crate::__::let_macros::value as let_statements;

pub trait Init<'i, 'r, S: Substrate>: Render<'r, S> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type Error;

	fn init(
		deps: &'i mut Waypoint<'i, dyn Repository>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl 'i + PinInit<Self, <Self as Init<'i, 'r, S>>::Error>;
}

pub trait InitAsync<'i, 'r, S: Substrate>: Render<'r, S> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type Error;

	fn init_async(
		deps: &'i mut Waypoint<'i, dyn Repository>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl 'i + Future<Output = impl 'i + PinInit<Self, <Self as InitAsync<'i, 'r, S>>::Error>>;
}

pub trait Render<'r, S: Substrate> {
	type RenderArgsBuilder: Default;
	type RenderArgs;
	type RenderContentBuilder: Default;
	type RenderContent;
	type RenderT;
	type Error;

	fn render(
		self: Pin<&'r mut Self>,
		ctx: S::RenderContext<'r>,
		args: Self::RenderArgs,
		content: Self::RenderContent,
	) -> Result<Self::RenderT, Self::Error>;
}

//TODO: Hide this somewhere.
pub trait ContentParent<'r, S: Substrate>: Render<'r, S> {
	type RenderAccumulator;

	fn start_accumulation(size_hint: SizeHint) -> Self::RenderAccumulator;
	fn finalize_accumulation(
		acc: Self::RenderAccumulator,
		ctx: S::RenderContext<'r>,
	) -> Result<Self::RenderT, Self::Error>;
}

pub trait ContentParentOf<'r, S: Substrate, T>: ContentParent<'r, S> {
	fn on_rendered(
		acc: &'_ mut Self::RenderAccumulator,
		ctx: S::RenderContext<'r>,
		rendered: T,
	) -> Result<(), Self::Error>;
}
