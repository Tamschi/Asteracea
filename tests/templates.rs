use asteracea::{
	bump_format, component, fragment,
	lignin_schema::lignin::{Attribute, Node},
};
use std::fmt::Display;

// Just for illustration purposes:
fn do_anything_here() {}
trait Select: Sized {
	fn select<R, S: FnOnce(Self) -> R>(self, selector: S) -> R;
}
impl<T> Select for T {
	fn select<R, S: FnOnce(Self) -> R>(self, selector: S) -> R {
		selector(self)
	}
}

component! {
	/// This is a maximal example for what Asteracea can currently do. A lot of the syntax here is optional.
	pub AVeryComplexComponent<A>

	// This where clause is applied to the component struct definition.
	where A: Display,

	/// After the struct itself, you define the constructor.
	/// Documentation and other attributes added here are applied to the generated `new` function.
	<D: Into<usize>>(a: A, d: D)

	/// Attributes defined here are applied to the `render` method.
	// What follows is the `render` method signature:
	<B> // #'bump is implicit if any HTML elements are present.
	(
		// &#self, is implicit.
		// #bump: &#'bump #asteracea::lignin_schema::lignin::bumpalo::Bump, is implicit if any HTML elements are present. Needed for any bump-allocated elements, i.e. HTML elements and formatted text.
		_b: B,
	) // Default: `-> #asteracea::lignin_schema::lignin::Node<#'bump>`

	//TODO: Inversion of control/"DI".
	/*ref for 'NEW, 'RENDER (
		a_extractable: AExtractable,
		b_extractable: BExtractable,
	)*/

	new with {
		// Constructor block.
	}

	// This is a top-level capture expression.
	// `c` is a field on the final struct, A its type (which can't be inferred) and a its value (which currently must be braced).
	|c: A = {a}|;

	/// You can use any number of top-level captures, which may be public and documented.
	/// The documentation is pasted above the resulting field declaration, which means it works as normal.
	/// (Outer documentation is a series of outer attributes, which means any outer attributes work here.)
	|null: usize = {d.into()}|;

	// Any node-producing part is valid here, but you need only one.
	// (This is pretty lenient. Any nodes that produce a value matching the render return type will work.)
	<div

		."Attributes-are-written"="like this." // This must appear directly inside HTML elements, above all children.

		//TODO: Support capturing directly in an attribute expression like so: .|...|
		.{Attribute {
			name: "an-attribute",
			value: "a value",
		}} // You can use Rust expressions here, too.
		// Attached access expressions work on attributes, but it's not necessary to clone this anymore.

		"You can add verbatim text like so."
		"This doesn't require direct `bump`, since text nodes are created by value."
		"Multiple strings aren't concatenated and will turn into distinct text nodes."

		{bump_format!("There's a handy macro for formatting text: {} {}", self.null, self.c)}

		{
			// Braces open a Rust child expression.
			// The contents are transcluded verbatim, and there's already a block so you can use statements like below.
			do_anything_here();

			// You can use the fragment! macro whenever you'd like to use an element outside a component! macro.
			// Note that captures aren't available here!
			fragment!{
				<span
					// | // Error: Captures are unavailable in this context: fragment!
				>.select(|x| x) // Attached access expressions also work on HTML elements.
			}
		 }.select(|x| x) // Attached access expressions work on Rust blocks too.

		/// Inner captures also exist.
		|
			//! Inner attributes are supported for all captures.
			//! This may be useful if the capture is very long.
			/// Inner attributes can be followed by outer ones here (but not the other way around, as per usual).
			/// It's really all pasted as a sequence of outer ones on the field declaration, though, so it's just cosmetic.
			#[allow(clippy::type_complexity)]
			_d: Vec<Vec<Vec<Vec<Vec<Vec<Vec<Vec<Vec<()>>>>>>>>>
			= {vec![vec![vec![vec![vec![vec![vec![vec![vec![()]]]]]]]]]}
		|;

		// Inner captures are perfect for child components.
		// Here you can see a shorthand for constructed captures (with the caveat that field type type parameters can't be inferred).
		// The general syntax is: |#field_name = #type::#constructor(#parameters)|
		pin |very_simple = VerySimple::new(&node, VerySimple::new_args_builder().build())?|.render(bump, VerySimple::render_args_builder().build())
		pin |very_simple_qualified = self::VerySimple::new(&node, VerySimple::new_args_builder().build())?|.render(bump, VerySimple::render_args_builder().build())
		// Note that the above lines end with .render() instead of ;.
		// This expands to the following call: self.#field_name.render().

		//TODO: static Part hoisting shorthand, probably needs to be
		//TODO: static mut Part for any Nodes though because those aren't Sync.

		// It's possible to:
		// - use or call the reference directly and/or
		// - call or access a differently named member and/or
		// - chain member access expressions as below:
		pin |very_simple_chained = VerySimple::new(&node, VerySimple::new_args_builder().build())?|
			.render(bump, VerySimple::render_args_builder().build()) // TODO: It should be possible to insert direct calls, that is `()` without leading `.identifier`, anywhere in this chain.
			.select(|x| x)

		// TODO: if {expression} Part [else Part], defaulting to nothing using an Into, returning a 'static empty Multi as Node<'_>.
		// TODO: for {} in {} Part via lignin::Node::Multi.

		<*VerySimple>
		<*Parametrized
			*new_arg = {"new arg value"}
			.render_arg = {"render arg value"}
		/Parametrized>

		<*VerySimple priv private_very_simple>
		<*VerySimple pub public_very_simple>

		{self.private_very_simple_pinned().render(bump, VerySimple::render_args_builder().build())}
		<*{self.public_very_simple_pinned()}>
	>
}

component! {
	pub VerySimple()() -> Node<'static>
	"Just text"
}

component! {
	Parametrized(priv new_arg: &'static str)
	(render_arg: &str)

	[
		"new_arg: " !{self.new_arg} <br>
		"render_arg: " !{render_arg}
	]
}

//TODO: This should show a warning for an unused struct. Reasearch why that doesn't happen.
component! {
	Unused()() // The render return type defaults to Node<'static> if bump isn't implicit.
	""
}

#[test]
fn test() {
	use asteracea::lignin_schema::lignin::bumpalo::Bump;
	use std::sync::Arc;

	enum RootTag {}
	let parent_node = Arc::new(rhizome::Node::new_for::<RootTag>());

	Box::pin(
		AVeryComplexComponent::<i32>::new(
			&parent_node,
			AVeryComplexComponentNewArgs::builder()
				.a(0)
				.d(0usize)
				.build(),
		)
		.unwrap(),
	)
	.as_ref()
	.render(
		&Bump::new(),
		AVeryComplexComponentRenderArgs::builder()._b(1).build(),
	);
}
