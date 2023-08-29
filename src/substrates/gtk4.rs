//! GTK4 substrate, using the [`gtk4`] crate.
#![allow(missing_docs)] //TODO

pub type Target<'a> = &'a Bump;
use bumpalo::Bump;

use self::__::{Attribute, AttributeName, AttributeValue, Element, ElementName};

#[derive(Debug)]
pub enum VdomNode<'a> {
	Multi(&'a [VdomNode<'a>]),
	Element {
		element: &'a Element<'a>,
		dom_binding: Option<()>,
	},
}

pub fn multi<'a>(nodes: &'a [VdomNode<'a>]) -> VdomNode<'a> {
	VdomNode::Multi(nodes)
}

pub fn schema_element<'a>(
	bump: &'a Bump,
	name: ElementName,
	attributes: &'a [Attribute<'a>], //TODO
	content: VdomNode<'a>,
	event_bindings: &'a [()], //TODO
) -> VdomNode<'a> {
	VdomNode::Element {
		element: &*bump.alloc_with(|| Element {
			name,
			attributes,
			content,
			event_bindings,
		}),
		dom_binding: None,
	}
}

pub fn attribute<'a>(name: AttributeName, value: AttributeValue<'a>) -> Attribute<'a> {
	Attribute { name, value }
}

pub mod schema {
	pub mod aspects {
		enum Vacant {}
		pub struct Attribute(Vacant);
	}

	pub mod elements {
		use crate::substrates::gtk4::__::ElementName;

		pub struct ApplicationWindow;

		impl ApplicationWindow {
			pub const TAG_NAME: ElementName = ElementName::ApplicationWindow;
		}
	}

	#[allow(non_camel_case_types)]
	pub mod attributes {
		use super::aspects::Attribute;
		use crate::substrates::gtk4::__::AttributeName;

		pub trait application<Aspect: ?Sized = Attribute> {
			const NAME: AttributeName = AttributeName::application;

			fn static_validate_on(_: Self)
			where
				Self: Sized,
			{
			}
		}

		impl application for super::elements::ApplicationWindow {}
	}
}

pub mod __ {
	use super::VdomNode;

	#[derive(Debug)]
	pub struct Element<'a> {
		pub name: ElementName,
		pub attributes: &'a [Attribute<'a>],
		pub content: VdomNode<'a>,
		pub event_bindings: &'a [()],
	}

	#[derive(Debug)]
	pub enum ElementName {
		ApplicationWindow,
	}

	#[derive(Debug)]
	pub struct Attribute<'a> {
		pub name: AttributeName,
		pub value: AttributeValue<'a>,
	}

	#[derive(Debug)]
	#[allow(non_camel_case_types)]
	pub enum AttributeName {
		application,
	}

	#[derive(Debug)]
	pub enum AttributeValue<'a> {
		Str(&'a str),
	}
}
