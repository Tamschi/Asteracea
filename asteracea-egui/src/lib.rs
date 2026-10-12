use std::convert::Infallible;

use asteracea::{Init, Render, Substrate};
use bon::Builder;
use egui::{Color32, Response, Ui};
use eros::AnyError;
use never_say_never::Never;

pub enum EguiSubstrate {}

impl Substrate for EguiSubstrate {
	type InitContext<'i> = ();
	type InitEDefault<'i> = AnyError;
	type RenderContext<'r> = &'r mut Ui;
	type RenderTDefault<'r> = Response;
	type RenderEDefault<'r> = AnyError;
}

impl EguiSubstrate {
	pub fn spinner(never: Never) -> Spinner {
		never
	}
}

#[derive(Default)]
pub struct BuildUnit;
impl BuildUnit {
	pub fn build(self) -> () {
		()
	}
}

#[derive(Default)]
pub struct NoContentBuilder;
impl NoContentBuilder {
	pub fn build(self) -> () {
		()
	}
}

macro_rules! token_init {
	($vis:vis $Name:ident) => {
		$vis struct $Name;

		impl<'i, 'r> Init<'i, 'r, EguiSubstrate> for $Name {
			type InitArgsBuilder = BuildUnit;
			type InitArgs = ();
			type InitT = Self;
			type InitE = Infallible;

			#[inline(always)]
			fn init(
				_: rhizome::sync::Node<std::any::TypeId, std::any::TypeId, rhizome::sync::DynValue>,
				_: <EguiSubstrate as Substrate>::InitContext<'i>,
				(): Self::InitArgs,
			) -> impl 'i + pinned_init::PinInit<Self::InitT, Self::InitE> {
				$Name
			}
		}
	};
}

#[derive(Builder)]
pub struct SpinnerArgs {
	/// Uses the style's `interact_size` in [`None`].
	size: Option<f32>,
	color: Option<Color32>,
}

impl Default for SpinnerArgsBuilder {
	fn default() -> Self {
		SpinnerArgs::builder()
	}
}

token_init!(pub Spinner);
impl<'r> Render<'r, EguiSubstrate> for Spinner {
	type RenderArgsBuilder = SpinnerArgsBuilder;
	type RenderArgs = SpinnerArgs;
	type RenderContentBuilder = NoContentBuilder;
	type RenderContent = ();
	type RenderT = Response;
	type RenderESet = ();

	#[inline(always)]
	fn render(
		self: std::pin::Pin<&'r mut Self>,
		ctx: <EguiSubstrate as Substrate>::RenderContext<'r>,
		Self::RenderArgs { size, color }: Self::RenderArgs,
		(): Self::RenderContent,
	) -> eros::Result<Self::RenderT, Self::RenderESet> {
		let mut spinner = egui::Spinner::new();
		if let Some(size) = size {
			spinner = spinner.size(size)
		}
		if let Some(color) = color {
			spinner = spinner.color(color)
		}
		Ok(ctx.add(spinner))
	}
}
