use std::{any::TypeId, future::Future, pin::Pin};

use eros::TypeSet;
use pinned_init::PinInit;
use rhizome::sync::{DynValue, Node};

pub trait Substrate {
	type InitContext<'i>;
	type InitEDefault<'i>;

	type RenderContext<'r>;
	type RenderTDefault<'r>;
	type RenderEDefault<'r>;
}

pub trait Init<'i, 'r, S: Substrate> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type InitT: Render<'r, S>;
	type InitE;

	fn init(
		deps: Node<TypeId, TypeId, DynValue>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl 'i + PinInit<Self::InitT, Self::InitE>;
}

pub trait AsyncInit<'i, 'r, S: Substrate> {
	type InitArgsBuilder: Default;
	type InitArgs;
	type InitT: Render<'r, S>;
	type InitE;

	fn init(
		deps: Node<TypeId, TypeId, DynValue>,
		ctx: S::InitContext<'i>,
		args: Self::InitArgs,
	) -> impl Future<Output = impl 'i + PinInit<Self::InitT, Self::InitE>>;
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
