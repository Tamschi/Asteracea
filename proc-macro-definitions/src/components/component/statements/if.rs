use loess::{grammar, quote_into_mixed_site, scaffold::SquareBrackets};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::keywords::{Else, If},
};
use loess_rust_opaque::ExpressionExceptStructExpression;
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::{StatementImpl, block::Block};

grammar! {
	pub struct BracketIf: PeekFrom, PopFrom {
		bracket_if: SquareBrackets<If>,
		conditions: ExpressionExceptStructExpression, //TODO: Use proper `Conditions`.
		then: Block,
		r#else: Option<(Else, Block)>,

	}
}

impl StatementImpl for BracketIf {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		// Always emits an accumulation.
		<dyn StatementImpl>::hint_size_one(self.bracket_if.span.join(), root, tokens);
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
		#![expect(unused_parens)] //FIXME?: `loess`'s quote grammar currently trips ofter `r#else` in some places.

		let BracketIf {
			bracket_if: SquareBrackets {
				span,
				contents: r#if,
			},
			conditions,
			then,
			r#else,
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
					//TODO?: Move the start of the accumulation into the branches to provide better size hints?
					let mut acc;
					{#(r#if, conditions)} {
						acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation({#({
								let mut tokens = TokenStream::new();
								then.hint_size(root, substrate, content_parent, attrs, &mut tokens);
								tokens
							})});
						{#({
							let mut render_proc = TokenStream::new();
							then.emit(
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
					} {#if let Some((r#else, else_block)) = (r#else) {
						{#(r#else)} {
							acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation({#({
								let mut tokens = TokenStream::new();
								else_block.hint_size(root, substrate, content_parent, attrs, &mut tokens);
								tokens
							})});
							{#({
								let mut render_proc = TokenStream::new();
								else_block.emit(
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
						}
					} else {
						else {
							acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation({#root}::SizeHint::ZERO);
						}
					}}
					<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)?
				}
			);
		});
	}
}
