use std::fmt::{self, Arguments, Write};

use asteracea::{
	SizeHint, components,
	substrate::{Accumulation, Substrate, SubstrateOnRender},
};
use inherent_associated_types::associate;

pub enum Html {}

impl Substrate for Html {
	type InitContext<'i> = ();
	type InitErrorDefault<'i> = !;
	type RenderContext<'r> = &'r mut dyn Write;
	type RenderTDefault<'r> = ();
	type RenderErrorDefault<'r> = fmt::Error;
}

impl<'r> Accumulation<'r> for Html {
	type RenderAccumulatorDefault = ();

	fn default_start_accumulation(_size_hint: SizeHint) -> Self::RenderAccumulatorDefault {}

	fn default_finalize_accumulation(
		acc: Self::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
	) -> Result<Self::RenderTDefault<'r>, Self::RenderErrorDefault<'r>> {
		Ok(acc)
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, !> for Html {
	#[inline(always)]
	fn on_rendered(
		_acc: &mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'_>,
		_rendered: !,
	) -> Result<(), std::fmt::Error> {
		Ok(())
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, ()> for Html {
	#[inline(always)]
	fn on_rendered(
		_acc: &mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'_>,
		_rendered: <Self as Substrate>::RenderTDefault<'r>,
	) -> Result<(), std::fmt::Error> {
		Ok(())
	}
}

impl<'r> SubstrateOnRender<'r, <Self as Accumulation<'r>>::RenderAccumulatorDefault, Arguments<'_>>
	for Html
{
	#[inline(always)]
	fn on_rendered(
		_acc: &mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		ctx: Self::RenderContext<'_>,
		rendered: Arguments<'_>,
	) -> Result<(), fmt::Error> {
		write!(ctx, "{}", rendered)
	}
}

/// Built-in components:
#[associate]
impl Html {
	pub type a = A;
	pub type div = Div;
}

components! {
	#![substrate = Html]

	pub A => ;

	pub Div => ;
}
