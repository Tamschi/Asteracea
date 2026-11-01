use asteracea::components;
use asteracea_epaint::EpaintSubstrate;

//TODO: Also support e.g. `for<S: Substrate>` (or any subtrait).
components!(
	#![substrate = EpaintSubstrate]

	pub RectShapeParent => {
		rect[size?, color?];
		rect[size?, color?];
	}
);
