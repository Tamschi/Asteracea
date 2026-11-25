use loess::grammar;
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::token::punct::{Colon, DotDot, Semi},
};
use loess_rust_opaque::Expression;
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct Transclusion: PeekFrom, PopFrom {
		pub dot_dot: DotDot,
		pub expression: Expression,
		pub semi: Semi,
	}
}

impl StatementImpl for Transclusion {
	fn hint_size(
		&self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		todo!("Transclusion hint_size")
	}

	fn boxed_emit(
		self: Box<Self>,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, Colon, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		self.emit(
			root,
			substrate,
			content_parent,
			additional_field_attrs,
			fields,
			attrs,
			init_proc,
			init_guards,
			render_proc,
		);
	}

	fn emit(
		self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, Colon, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		todo!("Transclusion emit")
	}
}
