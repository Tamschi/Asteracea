use loess::{
	grammar, quote_into_with_exact_span,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
	IntoTokens, SimpleSpanned,
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::keywords::{Async, Const},
	lex::token::punct::FatArrow,
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
		pub slot_args: Option<CurlyBraces<Optimistic<SlotArgs>>>,
		pub fat_arrow: FatArrow,
		pub body: Statement,
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
			slot_args,
			fat_arrow,
			body,
		} = self;

		// This (hopefully) enables unused function warnings.
		let init = Ident::new("init", name.span());
		let render = Ident::new("render", name.span());

		let mut fields = TokenStream::new();
		let mut init_proc = TokenStream::new();
		let mut render_proc = TokenStream::new();

		let content_parent = Ident::new("Self", name.span()).into_token_stream();
		body.emit(
			root,
			substrate,
			&content_parent,
			&mut fields,
			&mut init_proc,
			&mut render_proc,
		);

		quote_into_with_exact_span!(name.span(), root, tokens, {
			{#(attributes)}
			{#(&visibility)} struct {#(&name)} {
				{#(fields)}
			}

			{#mixed_site {
				{#if let Some(r#async) = r#async {
					{#located_at(r#async.span()) {
						#error("TODO: async components")
					}}
				} else {
					/// Synchronously initialised.
					impl<'i, 'r> {#root}::Init<'i, 'r, {#(substrate)}> for {#(&name)} {
						type InitArgsBuilder = ();
						type InitArgs = ();
						type InitESet = <{#(substrate)} as {#root}::Substrate>::InitESetDefault<'r>;

						fn {#(init)}(
							deps: {#root}::DepsNode,
							ctx: <{#(substrate)} as {#root}::Substrate>::InitContext<'i>,
							args: <Self as {#root}::Init<'r, 'r, {#(substrate)}>>::InitArgs,
						) -> impl 'i + {#root}::PinInit<
							Self,
							{#root}::ErrorUnion<<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitESet>,
						> {#located_at(fat_arrow.eq.span().join(fat_arrow.gt.span()).unwrap_or_else(|| fat_arrow.eq.span())){
							{
								let init = move |_this: *mut Self| -> {#root}::core::result::Result<
									(),
									{#root}::ErrorUnion<<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitESet>,
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
						{#root}::ErrorUnion<<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet>,
					> {
						let mut acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation();
						{#(render_proc)}
						<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)
					}
				}

				//TODO: Only if this can have content.
				#[doc(hidden)]
				impl<'r> {#root}::ContentParent<'r, {#(substrate)}> for {#(&name)} {
					type RenderAccumulator = <{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::RenderAccumulatorDefault;

					#[inline(always)]
					fn start_accumulation() -> Self::RenderAccumulator {
						<{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::default_start_accumulation()
					}

					#[inline(always)]
					fn finalize_accumulation(
						acc: Self::RenderAccumulator,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						{#root}::ErrorUnion<<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet>,
					> {
						<{#(substrate)} as {#root}::DefaultAccumulationSubstrate<'r>>::default_finalize_accumulation(acc, ctx)
					}
				}

				//TODO: Only if this can have content.
				#[doc = {#root}::concat!("[`", {#root}::stringify!({#(&name)}), "`] inherits accepted content from [`", {#root}::stringify!({#(substrate)}), "`].")]
				impl<'r, T> {#root}::ContentParentOf<'r, {#(substrate)}, T> for {#(name)}
				where
					{#(substrate)}: {#root}::ContentParentSubstrate<'r, <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator, T>,
				{
					#[inline(always)]
					fn on_rendered(
						acc: &'_ mut <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						rendered: T,
					) {
						<{#(substrate)} as {#root}::ContentParentSubstrate<'r, <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator, T>>::on_rendered(acc, ctx, rendered)
					}
				}
			}}
		});
	}
}
