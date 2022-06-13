#![allow(clippy::type_complexity)]

use asteracea::services::Invalidator;
use lignin::web::{Event, Materialize};
use std::sync::atomic::{AtomicUsize, Ordering};

asteracea::component! {
	/// A simple counter.
	pub(crate) Counter(
		priv dyn invalidator: dyn Invalidator,
		starting_count: usize,
	)(
		class?: &'bump str,
		button_text: &str = "Click me!",
	)

	<div
		.class?={class}

		let self.count = AtomicUsize::new(starting_count);
		!"This button was clicked {} times:"(self.count.load(Ordering::Relaxed))
		<button
			!(button_text)
			on bubble click = active Self::on_click
		>
	/div>
}

impl Counter {
	fn on_click(&self, event: Event) {
		// This needs work.
		let event: web_sys::Event = event.materialize();
		event.stop_propagation();

		self.count.fetch_add(1, Ordering::Relaxed);
		self.invalidator.invalidate_with_context(None);
	}

	pub fn count(&self) -> usize {
		self.count.load(Ordering::Relaxed)
	}

	pub fn increment(&self) {
		self.count.fetch_add(1, Ordering::Relaxed);
		self.invalidator.invalidate_with_context(None);
	}
}

asteracea::component! {
	Parent()()

	<*Counter priv counter *starting_count={0}>
}

#[test]
#[allow(warnings)]
fn use_component() {
	Box::pin(
		Parent::new(todo!(), Parent::new_args_builder().build())
			.unwrap()
			.0,
	)
	.as_ref()
	.render(todo!(), todo!());
}
