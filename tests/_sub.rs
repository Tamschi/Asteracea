use std::fmt;

use asteracea::{
	components,
	substrate::{
		let_statements::{
			Cell, CellCyclic, Computed, ComputedUncached, ComputedUncachedMut, Distinct,
			SignalCellTypes, SignalTypes,
		},
		Accumulation, Substrate, SubstrateOnRender,
	},
	SizeHint,
};
use flourish::{
	GlobalSignalsRuntime, Signal, SignalArcDyn, SignalArcDynCell, SignalDyn, SignalDynCell,
	SignalWeakDynCell,
};
use inherent_associated_types::associate;

pub struct Sub;

pub struct ErrorDefault;

impl Substrate for Sub {
	type InitContext<'i> = ();
	type InitErrorDefault<'i> = ErrorDefault;
	type RenderContext<'r> = ();
	type RenderTDefault<'r> = ();
	type RenderErrorDefault<'r> = ();
}

impl From<()> for ErrorDefault {
	fn from((): ()) -> Self {
		ErrorDefault
	}
}

impl From<!> for ErrorDefault {
	fn from(value: !) -> Self {
		value
	}
}

impl<'r> Accumulation<'r> for Sub {
	type RenderAccumulatorDefault = ();

	fn default_start_accumulation(_size_hint: SizeHint) -> Self::RenderAccumulatorDefault {
		// Do nothing.
	}

	fn default_finalize_accumulation(
		_acc: Self::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
	) -> Result<Self::RenderTDefault<'r>, Self::RenderErrorDefault<'r>> {
		// Do nothing.
		Ok(())
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, ()> for Sub {
	fn on_rendered(
		_acc: &'_ mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: (),
	) {
		// Do nothing.
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, !> for Sub {
	fn on_rendered(
		_acc: &'_ mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: !,
	) {
		// Do nothing.
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, &'static str>
	for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: &'static str,
	) {
		// Do nothing.
	}
}

impl<'r>
	SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, fmt::Arguments<'r>>
	for Sub
{
	fn on_rendered(
		_acc: &'_ mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
		_rendered: fmt::Arguments<'r>,
	) {
		// Do nothing.
	}
}

impl<T: 'static + Send> SignalTypes<T> for Sub {
	type SignalDyn = SignalDyn<'static, T, GlobalSignalsRuntime>;
	type SignalArcDyn = SignalArcDyn<'static, T, GlobalSignalsRuntime>;
}

impl<T: 'static + Send> SignalCellTypes<T> for Sub {
	type SignalDynCell = SignalDynCell<'static, T, GlobalSignalsRuntime>;
	type SignalArcDynCell = SignalArcDynCell<'static, T, GlobalSignalsRuntime>;
}

impl<T: 'static + Send> Cell<T> for Sub {
	fn cell(initial_value: T) -> Self::SignalArcDynCell {
		Signal::cell(initial_value).into_dyn_cell()
	}
}

//FIXME: Fix flourish to relax the `'static` bound on `F`.
impl<T: 'static + Send, F: 'static + FnOnce(&SignalWeakDynCell<T, GlobalSignalsRuntime>) -> T>
	CellCyclic<T, F> for Sub
{
	fn cell_cyclic(make_initial_value: F) -> Self::SignalArcDynCell {
		Signal::cell_cyclic(make_initial_value).into_dyn_cell()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> Computed<T, F> for Sub {
	fn computed(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + Sync + Fn() -> T> ComputedUncached<T, F> for Sub {
	fn computed_uncached(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed_uncached(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> ComputedUncachedMut<T, F> for Sub {
	fn computed_uncached_mut(fn_pin: F) -> Self::SignalArcDyn {
		Signal::computed_uncached_mut(fn_pin).into_dyn()
	}
}

impl<T: 'static + Send, F: 'static + Send + FnMut() -> T> Distinct<T, F> for Sub {
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
