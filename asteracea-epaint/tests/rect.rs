use asteracea::components;
use asteracea_epaint::Epaint;

//TODO: Also support e.g. `for<S: Substrate>` (or any subtrait).
components!(
	#![substrate = Epaint]

	pub RectShapeParent => {
		rect[size?, color?];
		rect[size?, color?];
	}
);
