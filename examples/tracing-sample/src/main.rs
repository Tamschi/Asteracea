use asteracea::{bumpalo::Bump, __::AnonymousContentParentParameters};
use core::pin::Pin;
use rhizome::Node;
use tracing::instrument;
use tracing_flame::FlameLayer;
use tracing_subscriber::prelude::*;

#[derive(Debug)]
struct A;
#[derive(Debug)]
struct B;
#[derive(Debug)]
struct C;
#[derive(Debug)]
struct D;
#[derive(Debug)]
struct E;

asteracea::component! {
	#[allow(clippy::needless_question_mark)] //FIXME
	ContainerComponent(
		// Constructor parameters:
		_plain: A,
		priv _stored: B,
		_optional?: C,
	)(
		// Render parameters:
		_plain: D,
		_optional?: u64 = 0,
		destructured @ E { .. }: E,
		..,
	) -> Sync

	..
}

fn main() {
	let _guard = set_up_tracing();
	render_components();
}

#[must_use]
fn set_up_tracing() -> impl Drop {
	let tree_layer = tracing_tree::HierarchicalLayer::default()
		.with_bracketed_fields(false)
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
	let app = ContainerComponent::new(
		&Node::new_for::<()>().into_arc(),
		ContainerComponent::new_args_builder()
			._plain(A)
			._stored(B)
			.build(),
	)
	.unwrap();
	let app = pin!(app).as_ref();

	let mut bump = Bump::new();

	print_app(&bump, app);
	bump.reset();
}

#[instrument(skip(bump, app))]
fn print_app(bump: &Bump, app: Pin<&ContainerComponent>) {
	let vdom = app
		.render(
			bump,
			ContainerComponent::render_args_builder()
				._plain(D)
				.destructured(E)
				.__Asteracea__anonymous_content((
					//FIXME: This should not be directly typed.
					AnonymousContentParentParameters {},
					Box::new(|_| Ok(asteracea::lignin::Node::Multi(&[]))),
				))
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
