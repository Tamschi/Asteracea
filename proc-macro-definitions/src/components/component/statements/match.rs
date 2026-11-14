use loess::{
	grammar, quote_into_mixed_site,
	scaffold::{CurlyBraces, Greedy, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{If, Match},
		token::punct::{Comma, FatArrow},
	},
};
use loess_rust_opaque::{Expression, ExpressionExceptStructExpression, Pattern};
use proc_macro2::{Group, TokenStream};

use crate::components::component::statements::{Statement, StatementImpl};

grammar! {
	pub struct BracketMatch: PeekFrom, PopFrom {
		bracket_match: SquareBrackets<Match>,
		scrutinee: ExpressionExceptStructExpression,
		body: CurlyBraces<Vec<MatchArm>>,
	}

	struct MatchArm: PopFrom {
		attrs: Greedy<Vec<OuterAttribute>>,
		pattern : Pattern,
		guard: Option<(If, Expression)>, //TODO: Support proper `MatchConditions` grammar!
		fat_arrow: FatArrow,
		statement: Statement,
		comma: Comma,
	}
}

impl StatementImpl for BracketMatch {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_one(self.bracket_match.span.join(), root, tokens);
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
		let BracketMatch {
			bracket_match: SquareBrackets {
				span,
				contents: r#match,
			},
			scrutinee,
			body: CurlyBraces {
				span: body_span,
				contents,
			},
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
					let mut acc;
					{#(r#match, scrutinee)} {#located_at(body_span.join()) {{
						{#for MatchArm {
							attrs,
							pattern,
							guard,
							fat_arrow,
							statement,
							comma,
						} in contents {
							{#for attr in attrs { {#(attr)} }}
							{#(pattern, guard, &fat_arrow)}
							{#located_at(fat_arrow.eq.span().join(fat_arrow.gt.span()).unwrap_or_else(|| fat_arrow.gt.span())) {{
								{
									acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation({#({
										let mut tokens = TokenStream::new();
										statement.hint_size(root, substrate, content_parent, &mut tokens);
										tokens
									})});
									{#({
										let mut render_proc = TokenStream::new();
										statement.emit(
											root,
											substrate,
											content_parent,
											&additional_field_attrs,
											fields,
											init_proc,
											init_guards,
											&mut render_proc,
										);
										render_proc
									})}
								}
							}}}
							{#(comma)}
						}}
					}}}
					<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)?
				}
			);
		});
	}
}
