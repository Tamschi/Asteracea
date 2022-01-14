use asteracea::bumpalo::Bump;
use rhizome::Node;
use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt, prelude::*};

asteracea::component! {
	FirstAndMaybeSecond()(
		first: &str,
		second?: &str,
	) -> Sync

	<ul
		//FIXME: Replace `{}` with `()` and make it optional.
		<li !"first = {first}"{}>
		<li !"second = {second:?}"{}>

		spread if {let Some(second) = second} <li
			"Nested:" <br>
			defer box <*FirstAndMaybeSecond .first={second}>
		>
	/ul>
}

fn main() {
	let _guard = set_up_tracing();
	render_components();
}

#[must_use]
fn set_up_tracing() -> impl Drop {
	let fmt_layer = fmt::layer();
	let (flame_layer, flush_guard) = FlameLayer::with_file("./tracing-sample.folded").unwrap();

	tracing_subscriber::registry()
		.with(fmt_layer)
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

	let bump = Bump::new();
	let vdom = app
		.render(
			&bump,
			FirstAndMaybeSecond::render_args_builder()
				.first("1")
				.second("2")
				.build(),
		)
		.unwrap();

	lignin_html::render_fragment(&vdom, &mut PrintWriter, 10).unwrap();
}

struct PrintWriter;
impl core::fmt::Write for PrintWriter {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		print!("{}", s);
		Ok(())
	}
}
