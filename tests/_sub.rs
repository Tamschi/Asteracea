use std::fmt;

use asteracea::{
	components,
	substrate::{
		ContentParentSubstrate, DefaultAccumulationSubstrate, SignalSubstrate,
		SignalSubstrateCellCyclic, SignalSubstrateComputed, SignalSubstrateComputedUncached,
		SignalSubstrateComputedUncachedMut, SignalSubstrateDistinct, Substrate,
	},
	SizeHint,
};
use eros::AnyError;
use flourish::{
	GlobalSignalsRuntime, Signal, SignalArcDyn, SignalArcDynCell, SignalDyn, SignalDynCell,
	SignalWeakDynCell,
};
use inherent_associated_types::associate;

pub struct Sub;

impl Substrate for Sub {
	type InitContext<'i> = ();
	type InitESetDefault<'i> = AnyError;
	type RenderContext<'r> = ();
	type RenderTDefault<'r> = ();
	type RenderESetDefault<'r> = AnyError;
}

impl<'r> DefaultAccumulationSubstrate<'r> for Sub {
	type RenderAccumulatorDefault = ();

	fn default_start_accumulation(_size_hint: SizeHint) -> Self::RenderAccumulatorDefault {
		// Do nothing.
	}

	fn default_finalize_accumulation(
		_acc: Self::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
	) -> eros::Result<Self::RenderTDefault<'r>, Self::RenderESetDefault<'r>> {
		// Do nothing.
		Ok(())
	}
}

impl<'r>
	ContentParentSubstrate<
		'r,
		<Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		(),
	> for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: (),
	) {
		// Do nothing.
	}
}

impl<'r>
	ContentParentSubstrate<
		'r,
		<Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		!,
	> for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: !,
	) {
		// Do nothing.
	}
}

impl<'r>
	ContentParentSubstrate<
		'r,
		<Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		&'static str,
	> for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: &'static str,
	) {
		// Do nothing.
	}
}

impl<'r>
	ContentParentSubstrate<
		'r,
		<Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		fmt::Arguments<'r>,
	> for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: fmt::Arguments<'r>,
	) {
		// Do nothing.
	}
}

impl<T: 'static + Send> SignalSubstrate<T> for Sub {
	type SignalDyn = SignalDyn<'static, T, GlobalSignalsRuntime>;
	type SignalDynCell = SignalDynCell<'static, T, GlobalSignalsRuntime>;
	type SignalArcDyn = SignalArcDyn<'static, T, GlobalSignalsRuntime>;
	type SignalArcDynCell = SignalArcDynCell<'static, T, GlobalSignalsRuntime>;

	fn cell(initial_value: T) -> Self::SignalArcDynCell {
		Signal::cell(initial_value).into_dyn_cell()
	}
}

//FIXME: Fix flourish to relax the `'static` bound on `F`.
impl<T: 'static + Send, F: 'static + FnOnce(&SignalWeakDynCell<T, GlobalSignalsRuntime>) -> T>
	SignalSubstrateCellCyclic<T, F> for Sub
{
	fn cell_cyclic(make_initial_value: F) -> Self::SignalArcDynCell {
		Signal::cell_cyclic(make_initial_value).into_dyn_cell()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> SignalSubstrateComputed<T, F> for Sub {
	fn computed(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + Sync + Fn() -> T> SignalSubstrateComputedUncached<T, F>
	for Sub
{
	fn computed_uncached(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed_uncached(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> SignalSubstrateComputedUncachedMut<T, F>
	for Sub
{
	fn computed_uncached_mut(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed_uncached_mut(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> SignalSubstrateDistinct<T, F> for Sub {
	fn distinct(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed(fn_pin).into_dyn()
	}
}

#[associate]
impl Sub {
	pub type div = Div;
	pub type span = Span;
}

components! {
	#![substrate = Sub]

	pub Div => ;
	pub Span => ;
}
