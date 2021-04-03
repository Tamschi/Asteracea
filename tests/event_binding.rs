asteracea::component! {
	pub Inline()() -> !Sync

	<button
		on capture click = fn on_click(self, _) {}
	>
}

asteracea::component! {
	pub Mvc()() -> !Sync

	<button
		on bubble click = active Self::on_click
	>
}

impl Mvc {
	fn on_click(self: std::pin::Pin<&Self>, _: lignin::web::Event) {}
}

asteracea::component! {
	pub Detached()() -> !Sync

	<button
		on bubble click = active detached
	>
}

// Both the signature with the pinned receiver and with a plain reference should work.
fn detached(_: &Detached, _: lignin::web::Event) {}
