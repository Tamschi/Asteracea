//! Substrate traits.\
//! These provide capabilities to [`components!`](`crate::components!`).

use std::ops::Deref;

use eros::TypeSet;

use crate::SizeHint;

pub mod bounds;

pub trait Substrate {
	type InitContext<'i>: Copy; //TODO: `Context: Copy` trait
	type InitESetDefault<'i>: TypeSet;

	type RenderContext<'r>: Copy; //TODO: `Context: Copy` trait
	type RenderTDefault<'r>;
	type RenderESetDefault<'r>: TypeSet;
}

pub trait SignalSubstrate<T>: Substrate {
	type SignalDyn: ?Sized;
	type SignalDynCell: ?Sized;
	type SignalArcDyn: Deref<Target = Self::SignalDyn>;
	type SignalArcDynCell: Deref<Target = Self::SignalDynCell>;

	fn cell(initial_value: T) -> Self::SignalArcDynCell;
}

pub trait SignalSubstrateCellCyclic<T, F>: SignalSubstrate<T> {
	fn cell_cyclic(make_initial_value: F) -> Self::SignalArcDynCell;
}

pub trait SignalSubstrateComputed<T, F>: SignalSubstrate<T> {
	fn computed(fn_pin: F) -> Self::SignalArcDyn;
}

pub trait SignalSubstrateComputedUncached<T, F>: SignalSubstrate<T> {
	fn computed_uncached(fn_pin: F) -> Self::SignalArcDyn;
}

pub trait SignalSubstrateComputedUncachedMut<T, F>: SignalSubstrate<T> {
	fn computed_uncached_mut(fn_pin: F) -> Self::SignalArcDyn;
}

pub trait SignalSubstrateDistinct<T, F>: SignalSubstrate<T> {
	fn distinct(fn_pin: F) -> Self::SignalArcDyn;
}

pub trait DefaultAccumulationSubstrate<'r>: Substrate {
	type RenderAccumulatorDefault;
	fn default_start_accumulation(size_hint: SizeHint) -> Self::RenderAccumulatorDefault;
	fn default_finalize_accumulation(
		acc: Self::RenderAccumulatorDefault,
		ctx: Self::RenderContext<'r>,
	) -> eros::Result<Self::RenderTDefault<'r>, Self::RenderESetDefault<'r>>;
}
pub trait ContentParentSubstrate<'r, RenderAccumulator, T>: Substrate {
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
