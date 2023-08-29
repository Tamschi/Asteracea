use std::mem::size_of;

use ::imgui::Ui;
use asteracea::{bumpalo::Bump, error::Escalation};

pub type VdomNode<'a> = ();

#[derive(Debug, Clone, Copy)]
pub struct Target<'a> {
	pub bump: &'a Bump,
	pub ui: &'a Ui,
}

impl<'a> Target<'a> {
	#[inline(always)]
	pub fn alloc_try_with<T>(
		self,
		f: impl FnOnce() -> Result<T, Escalation>,
	) -> Result<&'a T, Escalation> {
		self.bump.alloc_try_with(f).map(|v| &*v)
	}
}

pub fn multi<'a>(_: &'a [VdomNode<'a>]) -> VdomNode<'a> {}

pub fn text<'a>(target: Target<'a>, text: &'a str) -> VdomNode<'a> {
	target.ui.text_wrapped(text)
}

#[macro_export]
macro_rules! format_text {
	($target:expr, $($input:tt)*) => {
		$crate::imgui::text($target, asteracea::bumpalo::format!(in $target.bump, $($input)*).into_bump_str())
	};
}
pub use format_text;
