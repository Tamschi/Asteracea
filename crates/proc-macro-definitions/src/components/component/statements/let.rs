use loess::{
	IntoTokens, SimpleSpanned, grammar, quote_into_mixed_site,
	scaffold::{Parentheses, SquareBrackets, UpTo},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{self, Mut},
		token::punct::{At, Colon, Eq, Not, Semi},
	},
};
use loess_rust_opaque::Type;
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};

use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct Let: PeekFrom, PopFrom {
		r#let: Keyword,
		r#mut: Option<Mut>,
		pattern: UpTo<ColonOrEq>,
		r#type: Option<(Colon, Type)>, //TODO: (Try to) infer from expression if not given.
		eq: Eq,
		complex_macro: Option<(At, Ident, Not, Parentheses)>,
		rhs_rest: UpTo<Semi>,
		semi: Semi,
	}

	enum Keyword: PeekFrom, PopFrom {
		Let(keywords::Let),
		ParenLet(Parentheses<keywords::Let>),
		BracketedLet(SquareBrackets<keywords::Let>),
	} else unreachable!() as &str;

	enum ColonOrEq: PeekFrom {
		Colon(Colon),
		Eq(Eq),
	} else unreachable!();
}

impl IntoTokens for Keyword {
	fn into_tokens(self, root: &TokenStream, tokens: &mut impl Extend<TokenTree>) {
		match self {
			Keyword::Let(r#let) => r#let.into_tokens(root, tokens),
			Keyword::ParenLet(parens) => parens.into_tokens(root, tokens),
			Keyword::BracketedLet(brackets) => brackets.into_tokens(root, tokens),
		}
	}
}

impl Keyword {
	fn span(&self) -> Span {
		match self {
			Keyword::Let(r#let)
			| Keyword::ParenLet(Parentheses {
				contents: r#let, ..
			})
			| Keyword::BracketedLet(SquareBrackets {
				contents: r#let, ..
			}) => r#let.span(),
		}
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
		_content_parent: &TokenStream,
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
			pattern,
			r#type,
			eq,
			complex_macro,
			rhs_rest,
			semi,
		} = self;

		#[expect(unused_parens)]
		let is_bracketed = matches!((r#let), Keyword::BracketedLet(_));

		match (r#let, r#type, complex_macro) {
			(
				Keyword::Let(r#let)
				| Keyword::BracketedLet(SquareBrackets {
					contents: r#let, ..
				}),
				Some((colon, r#type)),
				Some((at, r#macro, not, Parentheses { span, contents })),
			) => {
				let field_name = Ident::new(
					&format!("__Asteracea__let_{}", fields.len()),
					r#let.span().resolved_at(Span::mixed_site()),
				);
				fields.push((attrs.to_owned(), Identifier(field_name.clone()), colon, {
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(at.at.span(), root, &mut tokens, {
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
				quote_into_mixed_site!(at.at.span(), root, init_proc, {
					let mut {#(&guard)} = {
						let value {#(&eq)} {#root}::let_macros::value::{#(&r#macro, &not)}{#located_at(span.join()) { (
							({#(contents)}),
							{#(substrate)},
						) }}{#(rhs_rest)};
						unsafe { {#root}::pinned_init_and_guard(value, &raw mut (*this).{#(&field_name)})? }
					};
					{#if !is_bracketed {
						{#(&r#let, &r#mut, &pattern, &eq)} {#root}::let_macros::access::{#(&r#macro, &not)}(
							*{#(&guard)},
							{#(substrate)},
							&{#(guard)},
							{#(&r#mut)},
						){#(&semi)}
					}}
				});

				quote_into_mixed_site!(at.at.span(), root, render_proc, {
					let let_borrow_guard = ();
					{#(r#let, &r#mut, pattern, eq)} {#root}::let_macros::access::{#(r#macro, not)}(
						&raw mut *(&mut *this).{#(field_name)},
						{#(substrate)},
						&let_borrow_guard,
						{#(r#mut)},
					){#(semi)}
				});
			}
			(
				Keyword::ParenLet(Parentheses {
					span: paren_span,
					contents: r#let,
				}),
				r#type,
				Some((at, r#macro, not, Parentheses { span, contents })),
			) => {
				//TODO: Redo this to `access`` the value off the stack (for consistent typing).
				quote_into_mixed_site!(paren_span.join(), root, init_proc, {
					{#(r#let, r#mut, pattern)}
					{#if let Some((colon, r#type)) = r#type {
						{#(colon)} &{#root}::let_macros::r#type::{#(&r#macro, &not)}(
							{#({
								let mut tokens = TokenStream::new();
								r#type.clone().into_tokens(root, &mut tokens);
								tokens
							})},
							{#(substrate)},
						)
					}}
					{#(eq)}
					&{#located_at(at.at.span()) {
						{#root}::let_macros::value::{#(&r#macro, &not)}{#located_at(span.join()) { (
							({#(contents)}),
							{#(substrate)},
						) }}
					}}
					{#(rhs_rest, semi)}
				});

				// Not rendered.
			}
			(
				Keyword::ParenLet(Parentheses {
					span,
					contents: r#let,
				}),
				r#type,
				None,
			) => quote_into_mixed_site!(span.join(), root, init_proc, {
				{#(r#let, r#mut, pattern, r#type, eq, rhs_rest, semi)}
			}),
			(
				Keyword::BracketedLet(SquareBrackets {
					span,
					contents: r#let,
				}),
				r#type,
				None,
			) => quote_into_mixed_site!(span.join(), root, render_proc, {
				{#(r#let, r#mut, pattern, r#type, eq, rhs_rest, semi)}
			}),
			(r#let, _, _) => quote_into_mixed_site!((r#let).span(), root, render_proc, {
				{{#error { {#root}::concat!("TODO: Unhandled `", {#root}::stringify!({#(r#let)}), "` form here.") }}}
			}),
		}
	}
}
