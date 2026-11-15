use std::{any::TypeId, future::Future, ops::Deref, pin::Pin};

use eros::{ErrorUnion, TypeSet};
use pinned_init::PinInit;
use rhizome::sync::{DynValue, NodeHandle};

use crate::SizeHint;

pub trait Substrate {
	type InitContext<'i>;
	type InitESetDefault<'i>: TypeSet;

	type RenderContext<'r>;
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
	fn cell_cyclic(fn_pin: F) -> Self::SignalArcDynCell;
}

pub trait SignalSubstrateComputed<T, F>: SignalSubstrate<T> {
	fn computed(fn_pin: F) -> Self::SignalArcDyn;
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

pub trait Init<'i, 'r, S: Substrate>: Render<'r, S> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type InitESet: TypeSet;

	fn init(
		deps: &'i mut NodeHandle<TypeId, TypeId, DynValue>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl 'i + PinInit<Self, ErrorUnion<Self::InitESet>>;
}

pub trait AsyncInit<'i, 'r, S: Substrate>: Render<'r, S> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type InitESet: TypeSet;

	fn init(
		deps: &'i mut NodeHandle<TypeId, TypeId, DynValue>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl Future<Output = impl 'i + PinInit<Self, ErrorUnion<Self::InitESet>>>;
}

pub trait Render<'r, S: Substrate> {
	type RenderArgsBuilder: Default;
	type RenderArgs;
	type RenderContentBuilder: Default;
	type RenderContent;
	type RenderT;
	type RenderESet: TypeSet;

	fn render(
		self: Pin<&'r mut Self>,
		ctx: S::RenderContext<'r>,
		args: Self::RenderArgs,
		content: Self::RenderContent,
	) -> eros::Result<Self::RenderT, Self::RenderESet>;
}

pub trait ContentParentSubstrate<'r, RenderAccumulator, T>: Substrate {
	fn on_rendered(acc: &'_ mut RenderAccumulator, ctx: Self::RenderContext<'r>, rendered: T);
}

//TODO: Hide this somewhere.
pub trait ContentParent<'r, S: Substrate>: Render<'r, S> {
	type RenderAccumulator;

	fn start_accumulation(size_hint: SizeHint) -> Self::RenderAccumulator;
	fn finalize_accumulation(
		acc: Self::RenderAccumulator,
		ctx: S::RenderContext<'r>,
	) -> eros::Result<Self::RenderT, Self::RenderESet>;
}

pub trait ContentParentOf<'r, S: Substrate, T>: ContentParent<'r, S> {
	fn on_rendered(acc: &'_ mut Self::RenderAccumulator, ctx: S::RenderContext<'r>, rendered: T);
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
