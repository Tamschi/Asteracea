//! DOM target. (TODO: Expand docs.)

use bumpalo::Bump;
use lignin::{Attribute, Element, ElementCreationOptions, EventBinding, ThreadBound};

pub use lignin_schema::html as schema;

pub type VdomNode<'a> = lignin::Node<'a, ThreadBound>;

pub fn text<'a>(text: &'a str) -> VdomNode<'a> {
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
