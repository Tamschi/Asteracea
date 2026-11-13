use loess::{grammar, quote_into_mixed_site, scaffold::SquareBrackets};
use loess_rust::{attributes::OuterAttribute, ident::Identifier, lex::keywords::While};
use loess_rust_opaque::ExpressionExceptStructExpression;
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::Block;

grammar! {
	pub struct BracketWhile: PeekFrom, PopFrom {
		pub bracket_while: SquareBrackets<While>,
		pub expression: ExpressionExceptStructExpression,
		pub block: Block,
	}
}

impl BracketWhile {
	pub fn emit(
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
		let BracketWhile {
			bracket_while: SquareBrackets {
				span,
				contents: r#while,
			},
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
					{#(r#while, expression, {
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
