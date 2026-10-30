use loess::{
	grammar, quote_into_with_exact_span,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
	IntoTokens, SimpleSpanned,
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::keywords::{Async, Const},
	vis::Visibility,
};
use proc_macro2::Group;
use proc_macro2::{Ident, TokenStream, TokenTree};
use statements::Statement;

use crate::components::component::args::{ConstructorArgs, RenderArgs, SlotArgs};

pub mod args;
pub mod statements;

grammar! {
	pub struct Component: PopFrom {
		pub attributes: Greedy<Vec<OuterAttribute>>,
		pub visibility: Option<Visibility>,
		pub r#const: Option<Const>,
		pub r#async: Option<Async>,
		pub name: Identifier,
		pub constructor_args: Option<Parentheses<Optimistic<ConstructorArgs>>>,
		pub render_args: Option<SquareBrackets<Optimistic<RenderArgs>>>,
		pub body: CurlyBraces<Vec<Statement>>,
	}
}

impl Component {
	pub fn emit(self, root: &TokenStream, tokens: &mut impl Extend<TokenTree>, substrate: &Group) {
		let Self {
			attributes,
			visibility,
			r#const,
			r#async,
			name,
			constructor_args,
			render_args,
			body,
		} = self;

		// This (hopefully) enables unused function warnings.
		let init = Ident::new("init", name.span());
		let render = Ident::new("render", name.span());

		quote_into_with_exact_span!(name.span(), root, tokens, {
			{#(attributes)}
			{#(&visibility)} struct {#(&name)} {}

			{#mixed_site {
				{#if let Some(r#async) = r#async {
					{#located_at(r#async.span()) {
						#error("TODO: async components")
					}}
				} else {
					impl<'i, 'r> {#root}::Init<'i, 'r, {#(substrate)}> for {#(&name)} {
						type InitArgsBuilder = ();
						type InitArgs = ();
						type InitT = Self;
						type InitE = ();

						fn {#(init)}(
							deps: {#root}::DepsNode,
							ctx: <{#(substrate)} as {#root}::Substrate>::InitContext<'i>,
							args: <Self as {#root}::Init<'r, 'r, {#(substrate)}>>::InitArgs,
						) -> impl 'i + {#root}::PinInit<
							<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitT,
							<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitE,
						> {
							let init = |_pointer| todo!();

							unsafe { {#root}::pinned_init::init_from_closure(init) }
						}
					}
				}}

				impl<'r> {#root}::Render<'r, {#(substrate)}> for {#(name)} {
					type RenderArgsBuilder = ();
					type RenderArgs = ();
					type RenderContentBuilder = ();
					type RenderContent = ();
					type RenderT = ();
					type RenderESet = ();
					
					fn render(
						self: {#root}::Pin<&'r mut Self>,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						args: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderArgs,
						content: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderContent,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet,
					> {
						todo!()
					}
				}
			}}
		});
	}
}
