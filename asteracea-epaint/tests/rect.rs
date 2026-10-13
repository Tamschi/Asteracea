use asteracea::components;
use asteracea_epaint::EpaintSubstrate;

//TODO: Also support e.g. `for<S: Substrate>` (or any subtrait).
components!(
	#![substrate = EguiSubstrate]

	pub RectShapeParent {
		rect[size?, color?];
	}
);
