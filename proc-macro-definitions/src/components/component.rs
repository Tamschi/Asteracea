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
use quote::ToTokens;
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
			body: CurlyBraces {
				span: body_span,
				contents: statements,
			},
		} = self;

		// This (hopefully) enables unused function warnings.
		let init = Ident::new("init", name.span());
		let render = Ident::new("render", name.span());

		let mut fields = TokenStream::new();
		let mut init_proc = TokenStream::new();
		let mut render_proc = TokenStream::new();

		let content_parent = Ident::new("Self", name.span()).into_token_stream();
		for statement in statements {
			statement.emit(
				root,
				substrate,
				&content_parent,
				&mut fields,
				&mut init_proc,
				&mut render_proc,
			);
		}

		quote_into_with_exact_span!(name.span(), root, tokens, {
			{#(attributes)}
			{#(&visibility)} struct {#(&name)} {

			}

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
						> {#located_at(body_span.join()){
							{
								let init = move |_this: *mut Self| -> {#root}::core::result::Result<
									(),
									<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitE,
								> {
									{#(init_proc)}
									todo!("init")
								};

								unsafe { {#root}::pinned_init::pin_init_from_closure(init) }
							}
						}}
					}
				}}

				impl<'r> {#root}::Render<'r, {#(substrate)}> for {#(&name)} {
					type RenderArgsBuilder = ();
					type RenderArgs = ();
					type RenderContentBuilder = ();
					type RenderContent = ();
					type RenderT = <{#(substrate)} as {#root}::Substrate>::RenderTDefault<'r>;
					type RenderESet = <{#(substrate)} as {#root}::Substrate>::RenderESetDefault<'r>;

					fn render(
						self: {#root}::Pin<&'r mut Self>,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						args: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderArgs,
						content: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderContent,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet,
					> {
						let mut acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation();
						{#(render_proc)}
						<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)
					}
				}

				//TODO: Only for default return type.
				impl<'r> {#root}::ContentParent<'r, {#(substrate)}> for {#(name)} {
					type RenderAccumulator = <{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault;
					fn start_accumulation() -> Self::RenderAccumulator {
						<{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::default_start_accumulation()
					}
					fn finalize_accumulation(
						acc: Self::RenderAccumulator,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet,
					> {
						<{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::default_finalize_accumulation(acc, ctx)
					}
				}
			}}
		});
	}
}
