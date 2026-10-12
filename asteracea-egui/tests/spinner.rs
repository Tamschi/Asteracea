use asteracea::components;
use asteracea_egui::EguiSubstrate;

//TODO: Also support e.g. `for<S: Substrate>` (or any subtrait).
components!(for EguiSubstrate {
	pub SpinnerParent[
		size?: f32,
		color?: Color32,
	] {
		spinner[size?=size, color?=color];
	}
});
