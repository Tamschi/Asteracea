use std::sync::Arc;

use asteracea::{
	SizeHint, components,
	substrate::{Accumulation, Substrate, SubstrateOnRender},
};
use epaint::{Brush, Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind};
use eros::AnyError;
use inherent_associated_types::associate;

pub enum Epaint {}

impl Substrate for Epaint {
	type InitContext<'i> = ();
	type InitESetDefault<'i> = AnyError;
	type RenderContext<'r> = &'r Style<'r>;
	type RenderTDefault<'r> = Shape;
	type RenderESetDefault<'r> = AnyError;
}

impl<'r> Accumulation<'r> for Epaint {
	type RenderAccumulatorDefault = Vec<Shape>;

	fn default_start_accumulation(size_hint: SizeHint) -> Self::RenderAccumulatorDefault {
		Vec::with_capacity(size_hint.0)
	}

	fn default_finalize_accumulation(
		acc: Self::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'r>,
	) -> eros::Result<Self::RenderTDefault<'r>, Self::RenderESetDefault<'r>> {
		Ok(Shape::Vec(acc))
	}
}

impl<'r>
	SubstrateOnRender<
		'r,
		<Self as Accumulation<'r>>::RenderAccumulatorDefault,
		<Self as Substrate>::RenderTDefault<'r>,
	> for Epaint
{
	#[inline(always)]
	fn on_rendered(
		acc: &mut <Self as Accumulation<'r>>::RenderAccumulatorDefault,
		_ctx: Self::RenderContext<'_>,
		rendered: <Self as Substrate>::RenderTDefault<'r>,
	) {
		acc.push(rendered)
	}
}

/// Built-in components:
#[associate]
impl Epaint {
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
	#![substrate = Epaint]

	pub CircleShape[
		center as Pos2 { x, y }: Pos2, //TODO: Infer type from pattern.
		radius as r: f32,
		fill: Color32 = Color32::TRANSPARENT, //TODO: Infer type from default.
		stroke: Stroke = Stroke::NONE, //TODO: Infer type from default.
	] => [{
		Shape::Circle(epaint::CircleShape {
			center: Pos2 { x, y },
			radius: r,
			fill,
			stroke,
		})
	}]

	pub RectShape[
		rect: Rect,
		corner_radius: CornerRadius = CornerRadius::ZERO,
		fill: Color32 = Color32::TRANSPARENT,
		stroke: Stroke = Stroke::NONE,
		stroke_kind: StrokeKind = StrokeKind::Inside,
		round_to_pixels?: bool,
		blur_width: f32 = 0.,
		brush?: Arc<Brush>,
		angle: f32 = 0.,
	] => [{Shape::Rect(epaint::RectShape {
		rect,
		corner_radius,
		fill,
		stroke,
		stroke_kind,
		round_to_pixels,
		blur_width,
		brush,
		angle,
	})}]
}
