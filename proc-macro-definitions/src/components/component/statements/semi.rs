use loess::grammar;
use loess_rust::{attributes::OuterAttribute, ident::Identifier};
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct Semi: PeekFrom, PopFrom {
		semi: loess_rust_lex::lex::token::punct::Semi,
	}
}

impl StatementImpl for Semi {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_zero(self.semi.semi.span(), root, tokens);
	}

	fn boxed_emit(
		self: Box<Self>,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
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
		_root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_additional_field_attrs: &[&OuterAttribute],
		_fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		_attrs: &[OuterAttribute],
		_init_proc: &mut TokenStream,
		_init_guards: &mut Vec<Identifier>,
		_render_proc: &mut TokenStream,
	) {
		// Nothing.
	}
}
