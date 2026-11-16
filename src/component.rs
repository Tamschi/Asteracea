//! Component traits.\
//! [`components!`](`crate::components!`) implements these for you.

use std::{any::TypeId, future::Future, pin::Pin};

use eros::{ErrorUnion, TypeSet};
use pinned_init::PinInit;
use rhizome::sync::{DynValue, NodeHandle};

use crate::{substrate::Substrate, SizeHint};

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
