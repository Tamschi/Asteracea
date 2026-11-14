use loess::grammar;

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
		root: &proc_macro2::TokenStream,
		substrate: &proc_macro2::Group,
		content_parent: &proc_macro2::TokenStream,
		attrs: &[loess_rust::attributes::OuterAttribute],
		tokens: &mut proc_macro2::TokenStream,
	) {
		self.statement
			.hint_size(root, substrate, content_parent, attrs, tokens);
	}

	fn boxed_emit(
		self: std::boxed::Box<Self>,
		root: &proc_macro2::TokenStream,
		substrate: &proc_macro2::Group,
		content_parent: &proc_macro2::TokenStream,
		additional_field_attrs: &[&loess_rust::attributes::OuterAttribute],
		fields: &mut Vec<(
			Vec<loess_rust::attributes::OuterAttribute>,
			loess_rust::ident::Identifier,
			proc_macro2::TokenStream,
		)>,
		attrs: &[loess_rust::attributes::OuterAttribute],
		init_proc: &mut proc_macro2::TokenStream,
		init_guards: &mut Vec<loess_rust::ident::Identifier>,
		render_proc: &mut proc_macro2::TokenStream,
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
		root: &proc_macro2::TokenStream,
		substrate: &proc_macro2::Group,
		content_parent: &proc_macro2::TokenStream,
		additional_field_attrs: &[&loess_rust::attributes::OuterAttribute],
		fields: &mut Vec<(
			Vec<loess_rust::attributes::OuterAttribute>,
			loess_rust::ident::Identifier,
			proc_macro2::TokenStream,
		)>,
		attrs: &[loess_rust::attributes::OuterAttribute],
		init_proc: &mut proc_macro2::TokenStream,
		init_guards: &mut Vec<loess_rust::ident::Identifier>,
		render_proc: &mut proc_macro2::TokenStream,
	) {
		todo!("Box emit")
	}
}
