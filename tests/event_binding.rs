#![allow(clippy::type_complexity)] //TODO: The macro should suppress this automatically.

asteracea::component! { substrate =>
	pub Inline()() -> Sync?

	<button
		on capture click = once fn (self, _) {}
		on error = fn on_error(self, _) {}
	>
}

asteracea::component! { substrate =>
	pub Mvc()() -> Sync

	<button
		on bubble click = active Self::on_click
	>
}

impl Mvc {
	fn on_click(&self, _: lignin::web::Event) {}
}

asteracea::component! { substrate =>
	pub MvcPinned()() -> !Sync

	<button
		on bubble click = active Self::on_click
	>
}

impl MvcPinned {
	fn on_click(self: std::pin::Pin<&Self>, _: lignin::web::Event) {}
}

asteracea::component! { substrate =>
	pub Detached()() -> Sync

	<button
		on bubble click = active detached1
		on bubble click = active detached2
		on bubble click = active detached3
	>
}

fn detached1(_: *const Detached, _: lignin::web::Event) {}
fn detached2(_: &Detached, _: lignin::web::Event) {}
fn detached3(_: std::pin::Pin<&Detached>, _: lignin::web::Event) {}
