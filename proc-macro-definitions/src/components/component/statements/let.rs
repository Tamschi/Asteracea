use loess::{IntoTokens, SimpleSpanned, grammar, quote_into_mixed_site, scaffold::Parentheses};
use loess_rust::{
	attributes::OuterAttribute,
	expr::block::BlockExpression,
	ident::Identifier,
	lex::{
		keywords::{Dyn, Else, Mut},
		token::punct::{Colon, Eq, Not, Semi},
	},
};
use loess_rust_opaque::{Expression, Type};
use proc_macro2::{Group, Ident, Span, TokenStream};

use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct ParenLet: PeekFrom, PopFrom {
		r#let: Parentheses<loess_rust_lex::lex::keywords::Let>,
		pat: (Option<Mut>, Identifier), //TODO: Allow pattern.
		r#type: Option<(Colon, Type)>,
		eq: Eq,
		rhs: (Expression, Option<(Else, BlockExpression)>),
		semi: Semi,
	}

	pub struct Let: PeekFrom, PopFrom {
		r#let: loess_rust_lex::lex::keywords::Let,
		r#mut: Option<Mut>,
		name: Identifier,
		r#type: (Colon, Type), //TODO: (Try to) infer from expression if not given.
		eq: Eq,
		rhs: (Ident, Not, Parentheses),
		semi: Semi,
	}

	enum Sugar: PopFrom, PeekFrom {
		Mut(Mut),
		Dyn(Dyn),
		None,
	} else unreachable!() as &str;
}

impl StatementImpl for ParenLet {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_zero(self.r#let.span.join(), root, tokens);
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
		let ParenLet {
			r#let: Parentheses {
				span,
				contents: r#let,
			},
			pat,
			r#type,
			eq,
			rhs,
			semi,
		} = self;

		// No fields.
		quote_into_mixed_site!(span.join(), root, init_proc, {
			{#(r#let, pat, r#type, eq, rhs, semi)}
		});
		// Not rendered.
	}
}

impl StatementImpl for Let {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_zero(self.r#let.span(), root, tokens);
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
		let Let {
			r#let,
			r#mut,
			name,
			r#type: (colon, r#type),
			eq,
			rhs: (r#macro, not, Parentheses { span, contents }),
			semi,
		} = self;

		let field_name = Ident::new(
			&format!("__Asteracea__let_{}", fields.len()),
			r#let.span().resolved_at(Span::mixed_site()),
		);
		fields.push((attrs.to_owned(), Identifier(field_name.clone()), colon, {
			let mut tokens = TokenStream::new();
			quote_into_mixed_site!((r#let.span()), root, &mut tokens, {
				{#root}::let_macros::r#type::{#(&r#macro, &not)}(
					{#({
						let mut tokens = TokenStream::new();
						r#type.clone().into_tokens(root, &mut tokens);
						tokens
					})},
					{#(substrate)},
				)
			});
			tokens
		}));

		let guard = Identifier(Ident::new(
			&format!("init_guard_{}", init_guards.len()),
			r#let.span().resolved_at(Span::mixed_site()),
		));
		init_guards.push(guard.clone());
		quote_into_mixed_site!((r#let.span()), root, init_proc, {
			let mut {#(&guard)} = {
				let value {#(&eq)} {#root}::let_macros::value::{#(&r#macro, &not)}{#located_at(span.join()) { (
					({#(contents)}),
					{#(substrate)},
				) }};
				unsafe { {#root}::pinned_init_and_guard(value, &raw mut (*this).{#(&field_name)})? }
			};
			{#(&r#let, &r#mut, &name, &eq)} {#root}::let_macros::access::{#(&r#macro, &not)}(
				*{#(&guard)},
				{#(substrate)},
				&{#(guard)},
				{#(&r#mut)},
			){#(&semi)}
		});

		quote_into_mixed_site!((r#let.span()), root, render_proc, {
			let let_borrow_guard = ();
			{#(r#let, &r#mut, name, eq)} {#root}::let_macros::access::{#(r#macro, not)}(
				&raw mut *(&mut *this).{#(field_name)},
				{#(substrate)},
				&let_borrow_guard,
				{#(r#mut)},
			){#(semi)}
		});
	}
}
