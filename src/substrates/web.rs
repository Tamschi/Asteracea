//! DOM target. (TODO: Expand docs.)

use bumpalo::Bump;
use lignin::{Attribute, Element, ElementCreationOptions, EventBinding, ThreadBound};

pub use lignin_schema::html as schema;

pub type Target<'a> = &'a Bump;
pub type VdomNode<'a> = lignin::Node<'a, ThreadBound>;

pub fn text<'a>(target: Target<'a>, text: &'a str) -> VdomNode<'a> {
	VdomNode::Text {
		text,
		dom_binding: None, //TODO: Add text dom binding support.
	}
}

pub fn multi<'a>(nodes: &'a [VdomNode<'a>]) -> VdomNode<'a> {
	VdomNode::Multi(nodes)
}

pub fn schema_element<'a>(
	bump: &'a Bump,
	name: &'a str,
	attributes: &'a [Attribute],
	content: VdomNode<'a>,
	event_bindings: &'a [EventBinding<ThreadBound>],
) -> VdomNode<'a> {
	//TODO: Add MathML and SVG support.
	VdomNode::HtmlElement {
		element: &*bump.alloc_with(|| Element {
			name,
			creation_options: ElementCreationOptions::new(), //TODO: Add `is` support.
			attributes,
			content,
			event_bindings,
		}),
		dom_binding: None, //TODO: Add DOM binding support.
	}
}

pub fn element_by_name<'a>(
	bump: &'a Bump,
	name: &'a str,
	attributes: &'a [Attribute],
	content: VdomNode<'a>,
	event_bindings: &'a [EventBinding<ThreadBound>],
) -> VdomNode<'a> {
	//TODO: Add MathML and SVG support.
	VdomNode::HtmlElement {
		element: &*bump.alloc_with(|| Element {
			name,
			creation_options: ElementCreationOptions::new(), //TODO: Add `is` support.
			attributes,
			content,
			event_bindings,
		}),
		dom_binding: None, //TODO: Add DOM binding support.
	}
}

pub fn attribute<'a>(name: &'a str, value: &'a str) -> Attribute<'a> {
	Attribute { name, value }
}

pub fn comment<'a>(_bump: &'a Bump, text: &'a str) -> VdomNode<'a> {
	VdomNode::Comment {
		comment: text,
		dom_binding: None, //TODO: Add DOM binding support.
	}
}

#[macro_export]
macro_rules! format_text {
	($target:expr, $($input:tt)*) => {
		$crate::substrates::web::text(
			$target,
			$crate::bumpalo::format!(in $target, $($input)*).into_bump_str(),
		)
	};
}
pub use format_text;

