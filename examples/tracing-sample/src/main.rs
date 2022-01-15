use asteracea::bumpalo::Bump;
use core::pin::Pin;
use rhizome::Node;
use tracing::instrument;
use tracing_flame::FlameLayer;
use tracing_subscriber::prelude::*;

asteracea::component! {
	FirstAndMaybeSecond()(
		recurse?: usize,
	) -> Sync

	<ul
		//FIXME: Replace `{}` with `()` and make it optional.
		<li !"recurse = {:?}"(recurse)>

		spread if {let Some(recurse) = recurse} <li
			spread if {recurse > 1} [
				"Nested with "<code ".recurse">":" <br>
				defer box <*FirstAndMaybeSecond .recurse={recurse - 1}>
			] else [
				"Nested without "<code ".recurse">":" <br>
				defer box <*FirstAndMaybeSecond>
			]
		>
	/ul>
}

fn main() {
	let _guard = set_up_tracing();
	render_components();
}

#[must_use]
fn set_up_tracing() -> impl Drop {
	let tree_layer = tracing_tree::HierarchicalLayer::default()
		.with_bracketed_fields(true)
		.with_wraparound(10); // Wraparound is helpful for deeply nested GUIs.
	let (flame_layer, flush_guard) = FlameLayer::with_file("./tracing-sample.folded").unwrap();

	tracing_subscriber::registry()
		.with(tree_layer)
		.with(flame_layer)
		.init();

	flush_guard
}

#[ergo_pin::ergo_pin]
fn render_components() {
	let app = FirstAndMaybeSecond::new(
		&Node::new_for::<()>().into_arc(),
		FirstAndMaybeSecond::new_args_builder().build(),
	)
	.unwrap();
	let app = pin!(app).as_ref();

	let mut bump = Bump::new();

	for recursion in 0..5 {
		print_app(&bump, app, recursion);
		bump.reset();
	}
}

#[instrument(skip(bump, app))]
fn print_app(bump: &Bump, app: Pin<&FirstAndMaybeSecond>, recursion: usize) {
	let vdom = app
		.render(
			bump,
			FirstAndMaybeSecond::render_args_builder()
				.recurse(recursion)
				.build(),
		)
		.unwrap();

	lignin_html::render_fragment(&vdom, &mut PrintWriter, 50).unwrap();
	println!();
}

struct PrintWriter;
impl core::fmt::Write for PrintWriter {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		print!("{}", s);
		Ok(())
	}
}
