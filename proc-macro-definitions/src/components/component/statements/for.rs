use loess::{
	grammar, quote_into_mixed_site,
	scaffold::{Parentheses, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::keywords::{For, In},
};
use loess_rust_opaque::{ExpressionExceptStructExpression, Pattern};
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::{StatementImpl, block::Block};

grammar! {
	pub struct BracketFor: PeekFrom, PopFrom {
		pub bracket_for: SquareBrackets<For>,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: Block,
	}

	pub struct ParenFor: PeekFrom, PopFrom {
		pub paren_for: Parentheses<For>,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: Block,
	}

	pub struct CachedFor: PeekFrom, PopFrom {
		pub r#for: For,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: Block,
	}
}

impl StatementImpl for BracketFor {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_unknown(self.bracket_for.span.join(), root, tokens);
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
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		let BracketFor {
			bracket_for: SquareBrackets {
				span,
				contents: r#for,
			},
			pattern,
			r#in,
			expression,
			block,
		} = self;

		let additional_field_attrs = additional_field_attrs
			.iter()
			.copied()
			.chain(attrs.iter())
			.collect::<Vec<_>>();

		quote_into_mixed_site!(span.join(), root, render_proc, {
			{#for attr in attrs { {#(attr)} }}
			<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
				&mut acc,
				ctx,
				{
					let mut acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation();
					{#(r#for, pattern, r#in, expression, {
						let mut render_proc = TokenStream::new();
						block.emit(
							root,
							substrate,
							content_parent,
							&additional_field_attrs,
							fields,
							&[],
							init_proc,
							init_guards,
							&mut render_proc,
						);
						render_proc
					})}
					<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)?
				}
			);
		});
	}
}
