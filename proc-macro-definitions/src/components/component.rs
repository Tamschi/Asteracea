use loess::{
	SimpleSpanned, grammar, quote_into_mixed_site, quote_into_with_exact_span,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{Async, Const},
		token::punct::FatArrow,
	},
	vis::Visibility,
};
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use statements::Statement;

use crate::components::component::args::{InitArg, InitArgs, RenderArgs, SlotArgs};

pub mod args;
pub mod statements;

grammar! {
	pub struct Component: PopFrom {
		pub attributes: Greedy<Vec<OuterAttribute>>,
		pub visibility: Option<Visibility>,
		pub r#const: Option<Const>,
		pub r#async: Option<Async>,
		pub name: Identifier,
		pub init_args: Option<Parentheses<Optimistic<InitArgs>>>,
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
			init_args,
			render_args,
			slot_args,
			fat_arrow,
			body,
		} = self;

		// This (hopefully) enables unused function warnings.
		let init = Ident::new("init", name.span());
		let render = Ident::new("render", name.span());

		let mut fields: Vec<(Vec<OuterAttribute>, Identifier, TokenStream)> = vec![];
		let mut init_proc = TokenStream::new();
		let mut init_guards: Vec<Identifier> = vec![];
		let mut render_proc = TokenStream::new();

		let content_parent = Ident::new("Self", name.span()).into_token_stream();
		body.emit(
			root,
			substrate,
			&content_parent,
			&[],
			&mut fields,
			&mut init_proc,
			&mut init_guards,
			&mut render_proc,
		);

		let (name_init_args_type, name_init_args_builder_type) = if init_args.is_some() {
			(
				[TokenTree::Ident(Ident::new(
					&format!("{}InitArgs", name.0.to_string().trim_start_matches("r#")),
					name.span(),
				))]
				.into_iter()
				.collect(),
				[TokenTree::Ident(Ident::new(
					&format!(
						"{}InitArgsBuilder",
						name.0.to_string().trim_start_matches("r#")
					),
					name.span(),
				))]
				.into_iter()
				.collect(),
			)
		} else {
			(
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoArgs
					});
					tokens
				},
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoArgsBuilder
					});
					tokens
				},
			)
		};

		let (name_render_args_type, name_render_args_builder_type) = if init_args.is_some() {
			(
				[TokenTree::Ident(Ident::new(
					&format!("{}RenderArgs", name.0.to_string().trim_start_matches("r#")),
					name.span(),
				))]
				.into_iter()
				.collect(),
				[TokenTree::Ident(Ident::new(
					&format!(
						"{}RenderArgsBuilder",
						name.0.to_string().trim_start_matches("r#")
					),
					name.span(),
				))]
				.into_iter()
				.collect(),
			)
		} else {
			(
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoArgs
					});
					tokens
				},
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoArgsBuilder
					});
					tokens
				},
			)
		};

		let (name_render_content_type, name_render_content_builder_type) = if init_args.is_some() {
			(
				[TokenTree::Ident(Ident::new(
					&format!(
						"{}RenderContent",
						name.0.to_string().trim_start_matches("r#")
					),
					name.span(),
				))]
				.into_iter()
				.collect(),
				[TokenTree::Ident(Ident::new(
					&format!(
						"{}RenderContentBuilder",
						name.0.to_string().trim_start_matches("r#")
					),
					name.span(),
				))]
				.into_iter()
				.collect(),
			)
		} else {
			(
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoContent
					});
					tokens
				},
				{
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(name.span(), root, &mut tokens, {
						{#root}::NoContentBuilder
					});
					tokens
				},
			)
		};

		quote_into_with_exact_span!(name.span(), root, tokens, {
			{#(attributes)}
			{#(&visibility)} struct {#(&name)} {
				{#for (attrs, ident, r#type) in fields.into_iter().rev() {
					{#(attrs)}
					{#(ident)}: {#(r#type)},
				}}
			}

			impl {#root}::Drop for {#(&name)} {
				fn drop(&mut self) {
					// Intentionally left empty. Rust will afterwards drop fields in declaration order,
					// i.e. in reverse of what's in the macro input.

					//TODO: Document this and check whether manual arrangement of fields improves performance.
					//TODO: In theory that's quite possible, as it could make access linear through nested components.
				}
			}

			{#if let Some(init_args) = init_args {
				#[{#root}::derive({#root}::bon::Builder)]
				#[builder(crate = {#root}::bon)]
				pub struct {#(&name_init_args_type)} {
					{#for (InitArg{ attrs, vis: _, r#dyn, identifier, colon, r#type }, comma) in init_args.contents.pairs() {
						{#if r#dyn.is_some() { {#continue;} }}
						{#for attr in attrs {
							{#(attr)}
							{#(identifier)}{#(colon)} {#(r#type)}{#(comma)}
						}}
					}}
				}
			}}

			{#mixed_site {
				{#if let Some(r#async) = r#async {
					{#located_at(r#async.span()) {
						#error("TODO: async components")
					}}
				} else {
					/// Synchronously initialised.
					impl<'i, 'r> {#root}::Init<'i, 'r, {#(substrate)}> for {#(&name)} {
						type InitArgsBuilder = {#(name_init_args_builder_type)};
						type InitArgs = {#(name_init_args_type)};
						type InitESet = <{#(substrate)} as {#root}::Substrate>::InitESetDefault<'r>;

						fn {#(init)}(
							deps: &'i mut {#root}::DepsNodeHandle,
							ctx: <{#(substrate)} as {#root}::Substrate>::InitContext<'i>,
							args: <Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitArgs,
						) -> impl 'i + {#root}::PinInit<
							Self,
							{#root}::ErrorUnion<<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitESet>,
						> {#located_at(fat_arrow.eq.span().join(fat_arrow.gt.span()).unwrap_or_else(|| fat_arrow.eq.span())){
							{
								let init = move |this: *mut Self| -> {#root}::core::result::Result<
									(),
									{#root}::ErrorUnion<<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitESet>,
								> {
									{#(init_proc)}
									{#root}::Ok({#root}::forget((
										{#for init_guard in init_guards.into_iter().rev() {
											{#(init_guard)},
										}}
									)))
								};

								unsafe { {#root}::pinned_init::pin_init_from_closure(init) }
							}
						}}
					}
				}}

				impl<'r> {#root}::Render<'r, {#(substrate)}> for {#(&name)} {
					type RenderArgsBuilder = {#(name_render_args_builder_type)};
					type RenderArgs = {#(name_render_args_type)};
					type RenderContentBuilder = {#(name_render_content_builder_type)};
					type RenderContent = {#(name_render_content_type)};
					type RenderT = <{#(substrate)} as {#root}::Substrate>::RenderTDefault<'r>;
					type RenderESet = <{#(substrate)} as {#root}::Substrate>::RenderESetDefault<'r>;

					fn {#(render)}(
						mut self: {#root}::Pin<&'r mut Self>,
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
