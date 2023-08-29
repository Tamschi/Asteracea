#![allow(non_snake_case)]

use crate::imgui;

asteracea::component! { imgui =>
	pub Button()(
		text: &str,
		clicked?: &mut bool,
	)

	{
		let c = bump.ui.button(text);
		if let Some(clicked) = clicked {
			*clicked = c
		}
	}
}

asteracea::component! { imgui =>
	pub Separator()()

	{
		bump.ui.separator()
	}
}

mod window;
pub use window::Window;
