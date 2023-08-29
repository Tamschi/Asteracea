#![allow(non_snake_case)]

use std::sync::atomic::{AtomicUsize, Ordering};

use ::imgui::Condition;
use asteracea_imgui::{
	components::{Button, Separator, Window},
	imgui,
};

static CHOICES: [&str; 2] = ["test test this is 1", "test test this is 2"];

asteracea::component! { imgui =>
	pub HellowWorld(
		value: usize = 0,
	)()

	let self.value = AtomicUsize::new(value);

	<*Window .title={"Hello world!"} .size={([300.0, 110.0], Condition::FirstUseEver)}

		"Hello, world!"
		"こんにちは世界！"

		with {
			let mut clicked = false;
		} [
			<*Button .text={CHOICES[self.value.load(Ordering::Relaxed)]} .clicked={&mut clicked}>
			{
				if clicked {
					self.value
					.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some((v + 1) % 2))
					.expect("Infallible");
				}
			}
		]

		<*Button .text={"This...is...imgui-rs!"}>
		<*Separator>

		with {
			let mouse_pos = bump.ui.io().mouse_pos;
		} !"Mouse Position: ({:.1},{:.1})"(mouse_pos[0], mouse_pos[1])
	>
}
