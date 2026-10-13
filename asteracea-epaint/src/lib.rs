use std::{convert::Infallible, pin::Pin, sync::Arc};

use asteracea::{Init, Render, Substrate, components};
use bon::Builder;
use epaint::{Brush, Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind};
use eros::AnyError;
use never_say_never::Never;

pub enum EpaintSubstrate {}

impl Substrate for EpaintSubstrate {
	type InitContext<'i> = ();
	type InitEDefault<'i> = AnyError;
	type RenderContext<'r> = &'r Style<'r>;
	type RenderTDefault<'r> = Shape;
	type RenderEDefault<'r> = AnyError;
}

impl EpaintSubstrate {
	pub fn circle(never: Never) -> CircleShape {
		never
	}

	pub fn rect(never: Never) -> RectShape {
		never
	}
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
	] {
		[{Shape::Circle(epaint::CircleShape {
			center,
			radius,
			fill,
			stroke,
		})}]
	}

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
	] {
		[{epaint::RectShape {
			rect,
			corner_radius,
			fill,
			stroke,
			stroke_kind,
			round_to_pixels,
			blur_width,
			brush,
			angle,
		}
		.into()}]
	}
}
