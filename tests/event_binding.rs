asteracea::component! {
	pub Inline()() -> !Sync

	<button
		on capture click = active fn on_click(self, event) {}
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

fn detached(_: std::pin::Pin<&Detached>, _: lignin::web::Event) {}
