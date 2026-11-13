use std::fmt;

use asteracea::{ContentParentSubstrate, DefaultAccumulationSubstrate, Substrate};
use eros::AnyError;

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

	fn default_start_accumulation() -> Self::RenderAccumulatorDefault {
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
