use asteracea::{
	ContentParentSubstrate, DefaultAccumulationSubstrate, Init, Render, Substrate, components,
};
use epaint::{Brush, Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind};
use eros::AnyError;
use inherent_associated_types::associate;

pub enum EpaintSubstrate {}

impl Substrate for EpaintSubstrate {
	type InitContext<'i> = ();
	type InitESetDefault<'i> = AnyError;
	type RenderContext<'r> = &'r Style<'r>;
	type RenderTDefault<'r> = Shape;
	type RenderESetDefault<'r> = AnyError;
}

impl<'r> DefaultAccumulationSubstrate<'r> for EpaintSubstrate {
	type RenderAccumulatorDefault = Vec<Shape>;

	fn default_start_accumulation() -> Self::RenderAccumulatorDefault {
		vec![]
	}

	fn default_finalize_accumulation(
		acc: Self::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
	) -> eros::Result<Self::RenderTDefault<'r>, Self::RenderESetDefault<'r>> {
		Ok(Shape::Vec(acc))
	}
}

impl<'r>
	ContentParentSubstrate<
		'r,
		<Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		<Self as Substrate>::RenderTDefault<'r>,
	> for EpaintSubstrate
{
	#[inline(always)]
	fn on_rendered(
		acc: &mut <Self as DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'_>,
		rendered: <Self as Substrate>::RenderTDefault<'r>,
	) {
		acc.push(rendered)
	}
}

/// Built-in components:
#[associate]
impl EpaintSubstrate {
	pub type circle = CircleShape;
	pub type rect = RectShape;
}

pub struct Style<'a> {
	parent: Option<&'a Self>,

	fill: Color32,
}

impl Style<'static> {
	pub const ROOT: Self = Self {
		parent: None,
		fill: todo!(),
	};
}

pub struct StyleDerivation {
	fill: Color32,
}

pub enum StyleDerivationValue<T> {
	Inherit,
	Initial,
	Set(T),
	Revert,
	RevertOne,
	Unset,
}
impl<T> From<T> for StyleDerivationValue<T> {
	fn from(value: T) -> Self {
		Self::Set(value)
	}
}

components! {
	#![substrate = EpaintSubstrate]

	pub CircleShape[
		center: Pos2,
		radius: f32,
		fill?: Color32,
		stroke?: Stroke,
	] => [{
		Shape::Circle(epaint::CircleShape {
			center: todo!(),
			radius: todo!(),
			fill: todo!(),
			stroke: todo!(),
		})
	}]

	pub RectShape[
		rect: Rect,
		corner_radius: CornerRadius,
		fill?: Color32,
		stroke?: Stroke,
		stroke_kind?: StrokeKind,
		round_to_pixels?: bool,
		blur_width?: f32,
		brush?: Arc<Brush>,
		angle?: f32,
	] => [{Shape::Rect(epaint::RectShape {
		rect: todo!(),
		corner_radius: todo!(),
		fill: todo!(),
		stroke: todo!(),
		stroke_kind: todo!(),
		round_to_pixels: todo!(),
		blur_width: todo!(),
		brush: todo!(),
		angle: todo!(),
	})}]
}
