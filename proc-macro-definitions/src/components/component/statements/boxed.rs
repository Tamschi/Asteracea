use loess::grammar;
use loess_rust::{attributes::OuterAttribute, ident::Identifier, lex::token::punct::Colon};
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::{StatementImpl, Storage};

grammar! {
	pub struct Box: PeekFrom, PopFrom {
		pub r#box: loess_rust_lex::lex::keywords::Box,
		pub storage: Option<Storage>,
		pub statement: std::boxed::Box<dyn StatementImpl>,
	}
}

impl StatementImpl for Box {
	fn hint_size(
		&self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		self.statement
			.hint_size(root, substrate, content_parent, attrs, tokens);
	}

	fn boxed_emit(
		self: std::boxed::Box<Self>,
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
		todo!("Box emit")
	}
}
