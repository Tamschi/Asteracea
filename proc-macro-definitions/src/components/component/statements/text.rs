use loess::{grammar, quote_into_mixed_site, scaffold::SquareBrackets};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::token::{
		literal::{AnyStringLiteral, RawStringLiteral, StringLiteral},
		punct::Semi,
	},
};
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::StatementImpl;

grammar! {
	/// Note: `format!` is not emitted if literal doesn't contain curly braces
	///        and `format_args` is [`None`].
	pub struct Text: PeekFrom, PopFrom {
		pub literal: AnyStringLiteral,
		pub format_args: Option<SquareBrackets>,
		pub semi: Semi,
	}
}

impl StatementImpl for Text {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_one(
			match &self.literal {
				AnyStringLiteral::Plain(StringLiteral(s))
				| AnyStringLiteral::Raw(RawStringLiteral(s)) => s.span(),
			},
			root,
			tokens,
		);
	}

	fn boxed_emit(
		self: std::boxed::Box<Self>,
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
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		_additional_field_attrs: &[&OuterAttribute],
		_fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		_init_proc: &mut TokenStream,
		_init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		let Text {
			literal,
			format_args,
			semi,
		} = self;

		// No fields.
		// No init_proc.

		let span = match &literal {
			AnyStringLiteral::Plain(l) => l.0.span(),
			AnyStringLiteral::Raw(l) => l.0.span(),
		};
		quote_into_mixed_site!(span, root, render_proc, {
			{#for attr in attrs { {#(attr)} }}
			<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
				&mut acc,
				ctx,
				{#if let Some(format_args) = format_args {
					{#located_at(format_args.span.join()) {
						{#root}::format_args!({#(literal)}, {#(format_args.contents)})
					}}
				} else {
					{#(literal)}
				}}
			){#(semi)}
		});
	}
}
