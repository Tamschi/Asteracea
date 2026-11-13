use std::ops::ControlFlow::Continue;

use loess::{
	Errors, Input, IntoTokens, PopFrom, SimpleSpanned, grammar, quote_into_mixed_site,
	scaffold::{CurlyBraces, Optimistic, Parentheses, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::Await,
		token::punct::{Dot, Semi},
	},
};
use loess_rust_opaque::Type;
use proc_macro2::{Group, Ident, Span, TokenStream};

use super::{Statement, Storage};

grammar! {
	pub struct Child: PopFrom {
		pub r#type: Type,
		pub new_args: Option<Parentheses>,
		pub dot_await: Option<(Dot, Await)>,
		pub render_args: Option<SquareBrackets>,
		pub storage: Option<Storage>,
		pub children: ChildChildren,
	}

	pub enum ChildChildren: PopFrom {
		Void(Semi),
		Braces(CurlyBraces<Optimistic<Vec<Statement>>>), //TODO: Named slots.
	} else "Expected `;` or `{`.";
}

impl Child {
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
		let Child {
			r#type,
			new_args,
			dot_await,
			render_args,
			storage,
			children,
		} = self;

		let (r#type, span) = (|| {
			let mut tokens = TokenStream::new();
			r#type.into_tokens(root, &mut tokens);
			let mut input = Input {
				tokens: tokens.into_iter().collect(),
				end: Span::mixed_site(),
			};
			let span = input.front_span();
			(
				if input.len() == 1
					&& let Continue(Some(identifier)) =
						Identifier::peek_pop_from(&mut input, &mut Errors::new())
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(identifier.span(), root, &mut tokens, {
						{#root}::Associated!({#(substrate)}[{#(identifier)}])
					});
					tokens
				} else {
					input.tokens.into_iter().collect()
				},
				span,
			)
		})();

		let field_name = proc_macro2::Ident::new(
			&format!("__Asteracea__child_{}", fields.len()),
			span.resolved_at(Span::mixed_site()),
		);
		fields.push((
			attrs.to_owned(),
			Identifier(field_name.clone()),
			r#type.clone(),
		));

		let mut content_guards: Vec<Identifier> = vec![];

		let mut as_init = TokenStream::new();
		quote_into_mixed_site!(span, root, &mut as_init, {
			<{#(&r#type)} as {#root}::{#if dot_await.is_some() {
				AsyncInit
			} else {
				Init
			}}<'_, 'r, {#(substrate)}>>
		});
		let as_init = as_init;
		let mut as_render = TokenStream::new();
		quote_into_mixed_site!(span, root, &mut as_render, {
			<{#(r#type)} as {#root}::Render<'_, {#(substrate)}>>
		});
		let as_render = as_render;
		quote_into_mixed_site!(span, root, init_proc, {
			let {#({
				let guard = Identifier(Ident::new( &format!("init_guard_{}", init_guards.len()), span.resolved_at(Span::mixed_site())));
				init_guards.push(guard.clone());
				guard
			})} = {
				let mut deps = {#root}::Clone::clone(&*deps);
				let child_init = {
					{#(&as_init)}::init(
						&mut deps,
						ctx,
						<{#(as_init)}::InitArgsBuilder as {#root}::Default>::default()
							//TODO
							.build(), //TODO: Should the build call go through a trait, too?
					)
				};
				let child_guard = unsafe {
					{#root}::pinned_init_and_guard(child_init, &raw mut (*this).{#(&field_name)})?
				};

				//TODO: Init content children.

				(
					{#for guard in content_guards.into_iter().rev() {
						{#(guard)},
					}}
					child_guard,
				)
			};
		});

		quote_into_mixed_site!(span, root, render_proc, {
			<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
				&mut acc,
				ctx,
				{#(&as_render)}::render(
					unsafe { {#root}::Pin::new_unchecked(&mut self.{#(field_name)}) },
					ctx,
					<{#(&as_render)}::RenderArgsBuilder as {#root}::Default>::default()
						//TODO
						.build(),
					<{#(as_render)}::RenderContentBuilder as {#root}::Default>::default()
						//TODO
						.build(),
				)?
			);
			//TODO
		});
	}
}
