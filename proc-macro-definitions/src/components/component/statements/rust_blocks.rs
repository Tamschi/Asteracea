use loess::{
	grammar, quote_into_mixed_site,
	scaffold::{Parentheses, SquareBrackets},
};
use loess_rust::{attributes::OuterAttribute, ident::Identifier, lex::token::delim::CurlyBraces};
use proc_macro2::TokenStream;

use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct ParenBrace: PeekFrom, PopFrom {
		paren_brace: Parentheses<CurlyBraces>,
	}

	pub struct BracketBrace: PeekFrom, PopFrom {
		paren_brace: SquareBrackets<CurlyBraces>,
	}
}

impl StatementImpl for ParenBrace {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &proc_macro2::Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_zero(self.paren_brace.span.join(), root, tokens);
	}

	fn boxed_emit(
		self: Box<Self>,
		root: &TokenStream,
		substrate: &proc_macro2::Group,
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
		root: &TokenStream,
		_substrate: &proc_macro2::Group,
		_content_parent: &TokenStream,
		_additional_field_attrs: &[&OuterAttribute],
		_fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		_init_guards: &mut Vec<Identifier>,
		_render_proc: &mut TokenStream,
	) {
		let ParenBrace {
			paren_brace:
				Parentheses {
					span,
					contents: CurlyBraces {
						span: inner_span,
						contents,
					},
				},
		} = self;

		// No fields.
		quote_into_mixed_site!(span.join(), root, init_proc, {
			{#for attr in attrs { {#(attr)} }}
			{#located_at(inner_span.join()) {
				{ {#(contents)} }
			}}
		})
		// Not rendered.
	}
}

impl StatementImpl for BracketBrace {
	fn hint_size(
		&self,
		root: &TokenStream,
		substrate: &proc_macro2::Group,
		content_parent: &TokenStream,
		attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		quote_into_mixed_site!(self.paren_brace.span.join(), root, tokens, {
			{#root}::SizeHint(0, {#root}::Some(1))
		});
	}

	fn boxed_emit(
		self: std::boxed::Box<Self>,
		root: &TokenStream,
		substrate: &proc_macro2::Group,
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
		root: &TokenStream,
		substrate: &proc_macro2::Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		let BracketBrace {
			paren_brace:
				SquareBrackets {
					span,
					contents: CurlyBraces {
						span: inner_span,
						contents,
					},
				},
		} = self;

		// No fields.
		// No init.
		quote_into_mixed_site!(span.join(), root, render_proc, {
			{#for attr in attrs { {#(attr)} }}
			<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
				&mut acc,
				ctx,
				{#located_at(inner_span.join()) {
					{ {#(contents)} }
				}}
			);
		})
	}
}
