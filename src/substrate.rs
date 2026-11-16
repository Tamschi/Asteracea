//! Substrate traits.\
//! These provide capabilities to [`components!`](`crate::components!`).

use crate::SizeHint;

pub mod bounds;

/// Required for all substrates. Informs signatures.
pub trait Substrate {
	type InitContext<'i>: Copy; //TODO: `Context: Copy` trait
	type InitErrorDefault<'i>;

	type RenderContext<'r>: Copy; //TODO: `Context: Copy` trait
	type RenderTDefault<'r>;
	type RenderErrorDefault<'r>;
}

pub mod let_statements;

pub trait Accumulation<'r>: Substrate {
	type RenderAccumulatorDefault;
	fn default_start_accumulation(size_hint: SizeHint) -> Self::RenderAccumulatorDefault;
	fn default_finalize_accumulation(
		acc: Self::RenderAccumulatorDefault,
		ctx: Self::RenderContext<'r>,
	) -> Result<Self::RenderTDefault<'r>, Self::RenderErrorDefault<'r>>;
}

pub trait SubstrateOnRender<'r, RenderAccumulator, T>: Substrate {
	fn on_rendered(acc: &'_ mut RenderAccumulator, ctx: Self::RenderContext<'r>, rendered: T);
}

// /// Units are always supported, strictly via no-op accumulation.
// impl<'r, P: Render<'r, S>, S: Substrate> ContentParentOf<'r, S, ()> for P
// where
// 	P: ContentParent<'r, S>,
// {
// 	#[inline(always)]
// 	fn on_rendered(
// 		_acc: &'_ mut Self::RenderAccumulator,
// 		_ctx: <S as Substrate>::RenderContext<'r>,
// 		_rendered: (),
// 	) {
// 		// Do nothing.
// 	}
// }
