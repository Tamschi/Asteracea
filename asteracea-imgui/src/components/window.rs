#![allow(non_snake_case)]

use ::imgui::Condition;
use asteracea::{error::Escalation, __::Built};

use crate::imgui::{self, Target, VdomNode};

pub struct WindowParentParameters {}
impl Built for WindowParentParameters {
	type Builder = Self;

	fn builder() -> Self::Builder {
		Self {}
	}
}
impl WindowParentParameters {
	pub fn build(self) -> Self {
		self
	}
}

asteracea::component! { imgui =>
	pub Window()(
		title: &str,
		size: ([f32; 2], Condition),
		children: Vec<(
			WindowParentParameters,
			Box<dyn 'a + FnOnce(Target<'bump>) -> Result<VdomNode<'bump>, Escalation>>,
		)>
	)

	{
		bump.ui.window(title).size(size.0, size.1).build(|| {
			//TODO: Allow declaring this as mutable in parameters.
			let mut children = children;
			for child in children.drain(..) {
				child.1(bump)?
			}
			Ok::<_, Escalation>(())
		}).unwrap_or_else(|| Ok(()))?
	}
}

impl<'render, 'a, 'bump>
	WindowRenderArgsBuilder<'render, 'a, 'bump, ((&'a str,), (([f32; 2], Condition),), ())>
{
	pub fn __Asteracea__anonymous_content(
		self,
		child: (
			WindowParentParameters,
			Box<dyn 'a + FnOnce(Target<'bump>) -> Result<VdomNode<'bump>, Escalation>>,
		),
	) -> WindowRenderArgsBuilder<
		'render,
		'a,
		'bump,
		(
			(&'a str,),
			(([f32; 2], Condition),),
			(
				Vec<(
					WindowParentParameters,
					Box<dyn 'a + FnOnce(Target<'bump>) -> Result<VdomNode<'bump>, Escalation>>,
				)>,
			),
		),
	> {
		self.children(vec![child])
	}
}

impl<'render, 'a, 'bump>
	WindowRenderArgsBuilder<
		'render,
		'a,
		'bump,
		(
			(&'a str,),
			(([f32; 2], Condition),),
			(
				Vec<(
					WindowParentParameters,
					Box<dyn 'a + FnOnce(Target<'bump>) -> Result<(), Escalation>>,
				)>,
			),
		),
	>
{
	pub fn __Asteracea__anonymous_content(
		mut self,
		child: (
			WindowParentParameters,
			Box<dyn 'a + FnOnce(Target<'bump>) -> Result<(), Escalation>>,
		),
	) -> Self {
		self.fields.2 .0.push(child);
		self
	}
}
