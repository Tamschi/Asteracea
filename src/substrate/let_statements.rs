use std::borrow::Borrow;

use crate::substrate::Substrate;

/// Provides signal types.
pub trait SignalTypes<T> {
	type SignalDyn: ?Sized;
	type SignalArcDyn: Borrow<Self::SignalDyn>;
}

/// Provides signal cell types.
pub trait SignalCellTypes<T> {
	type SignalDynCell: ?Sized;
	type SignalArcDynCell: Borrow<Self::SignalDynCell>;
}

/// Provides [`@cell!`][`crate::component::let_statements::cell!`].
pub trait Cell<T>: SignalCellTypes<T> {
	fn cell(initial_value: T) -> Self::SignalArcDynCell;
}

/// Provides [`@cell_cyclic!`][`crate::component::let_statements::cell_cyclic!`].
pub trait CellCyclic<T, F>: SignalCellTypes<T> {
	fn cell_cyclic(make_initial_value: F) -> Self::SignalArcDynCell;
}

/// Provides [`@computed!`][`crate::component::let_statements::computed!`].
pub trait Computed<T, F>: SignalTypes<T> {
	fn computed(fn_pin: F) -> Self::SignalArcDyn;
}

/// Provides [`@computed_uncached!`][`crate::component::let_statements::computed_uncached!`].
pub trait ComputedUncached<T, F>: SignalTypes<T> {
	fn computed_uncached(fn_pin: F) -> Self::SignalArcDyn;
}

/// Provides [`@computed_uncached_mut!`][`crate::component::let_statements::computed_uncached_mut!`].
pub trait ComputedUncachedMut<T, F>: SignalTypes<T> {
	fn computed_uncached_mut(fn_pin: F) -> Self::SignalArcDyn;
}

/// Provides [`@distinct!`][`crate::component::let_statements::distinct!`].
pub trait Distinct<T, F>: SignalTypes<T> {
	fn distinct(fn_pin: F) -> Self::SignalArcDyn;
}
