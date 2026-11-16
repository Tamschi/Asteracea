use loess::{
	SimpleSpanned, grammar, quote_into_mixed_site, quote_into_with_exact_span,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{Async, Const, Where},
		token::punct::{Colon, FatArrow},
	},
	vis::Visibility,
};
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use statements::Statement;

use crate::components::component::args::{InitArg, InitArgs, RenderArg, RenderArgs, SlotArgs};

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
	pub fn emit(
		self,
		root: &TokenStream,
		tokens: &mut impl Extend<TokenTree>,
		substrate: &Group,
		substrate_where: &Option<(Where, TokenStream)>,
	) {
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

		let mut fields: Vec<(Vec<OuterAttribute>, Identifier, Colon, TokenStream)> = vec![];
		let mut init_proc = TokenStream::new();
		let mut init_guards: Vec<Identifier> = vec![];
		let mut body_size_hint = TokenStream::new();
		let mut render_proc = TokenStream::new();

		let content_parent = Ident::new("Self", name.span()).into_token_stream();

		body.hint_size(root, substrate, &content_parent, &mut body_size_hint);
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

		let (name_render_args_type, name_render_args_builder_type) = if render_args.is_some() {
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

		let (name_render_content_type, name_render_content_builder_type) = if slot_args.is_some() {
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
				// According to <https://stackoverflow.com/questions/51933660/xeon-cpu-e5-2603-backward-memory-prefetch>,
				// some CPUs (at least sometimes) perform much better when reading memory forwards. As such, the fields
				// are stored in order here, and only dropping is reversed.
				{#for (attrs, ident, colon, r#type) in fields.iter() {
					{#(attrs)}
					{#(ident, colon)} {#root}::ManuallyDrop<{#(r#type)}>,
				}}
			}

			impl {#root}::Drop for {#(&name)} {
				fn drop(&mut self) {
					unsafe {
						// Drop in reverse:
						{#for (attrs, ident, _, _) in fields.into_iter().rev() {
							{#(attrs)}
								{#root}::ManuallyDrop::<_>::drop(&mut self.{#(ident)});
						}}
					}
				}
			}

			{#mixed_site {
				{#if let Some(Parentheses { span, contents }) = &init_args {
					{#located_at(span.join()) {
						#[{#root}::derive({#root}::bon::Builder)]
						#[builder(crate = {#root}::bon)]
						{#(&visibility)} struct {#(&name_init_args_type)} {
							{#for (InitArg{ attrs, vis: _, r#mut: _, identifier, expansion, question, r#type, default }, comma) in contents.args.pairs() {
								{#for attr in attrs { {#(attr)} }}
								{#if let Some((eq, default)) = default {
									#[builder(default {#(eq, default)})]
								}}
								{#(identifier)}
								{#if let Some((colon, _)) = &r#type {
									{#(colon)}
								} else {
									{#located_at(identifier.span()) {
										:
									}}
								}}
								{#if let Some(question) = question { {#located_at(question.question.span()) { {#root}::Option< }}}}
								{#if let Some((_, r#type)) = r#type {
									{#(r#type)}
								} else {
									{#if let Some(r#type) = expansion.as_ref().and_then(|(_, pattern)| args::infer_pattern_type(&pattern)).or_else(|| default.as_ref().and_then(|(_, expr)| args::infer_expression_type(&expr))) {
										{#(r#type)}
									} else {
										{#error{ {#root}::concat!("Could not infer type of `", {#root}::stringify!({#(&name)}), "`'s init parameter `", {#root}::stringify!({#(identifier)}), "` from expansion (`as …`) or default (`= …`).") }}
									}}
								}}
								{#if let Some(question) = question { {#located_at(question.question.span()) { > }}}}
								{#(comma)}
							}}
						}

						impl {#root}::Default for {#(&name_init_args_builder_type)} {
							fn default() -> Self {
								{#(&name_init_args_type)}::builder()
							}
						}
					}}
				}}

				{#if let Some(r#async) = r#async {
					{#located_at(r#async.span()) {
						#error("TODO: async components")
					}}
				} else {
					/// Synchronously initialised.
					impl<
						'i,
						'r,
						{#if substrate_where.is_some() { {#(substrate)}, }}
					> {#root}::Init<'i, 'r, {#(substrate)}> for {#(&name)} {#(substrate_where)} {
						type InitArgsBuilder = {#(name_init_args_builder_type)};
						type InitArgs = {#(&name_init_args_type)};
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
									{#if let Some(Parentheses{span, contents}) = init_args {
										{#located_at(span.join()) {
											let {#(name_init_args_type)} {
												{#for (InitArg { r#mut, identifier, expansion, .. }, comma) in contents.args.pairs() {
													{#(r#mut, identifier)}
													{#if let Some((r#as, pattern)) = expansion {
														{#located_at(r#as.span()) {
															:
														}}
														{#(pattern)}
													}}
													{#(comma)}
												}}
											} = args;

										}}
									}}

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

				{#if let Some(SquareBrackets { span, ref contents }) = render_args {
					{#located_at(span.join()) {
						#[{#root}::derive({#root}::bon::Builder)]
						#[builder(crate = {#root}::bon)]
						{#(&visibility)} struct {#(&name_render_args_type)} {
							{#for (RenderArg{ attrs, r#mut: _, identifier, expansion, question, r#type, default }, comma) in contents.pairs() {
								{#for attr in attrs { {#(attr)} }}
								{#if let Some((eq, default)) = default {
									#[builder(default {#(eq, default)})]
								}}
								{#(identifier)}
								{#if let Some((colon, _)) = &r#type {
									{#(colon)}
								} else {
									{#located_at(identifier.span()) {
										:
									}}
								}}
								{#if let Some(question) = question { {#located_at(question.question.span()) { {#root}::Option< }}}}
								{#if let Some((_, r#type)) = r#type {
									{#(r#type)}
								} else {
									{#if let Some(r#type) = expansion.as_ref().and_then(|(_, pattern)| args::infer_pattern_type(&pattern)).or_else(|| default.as_ref().and_then(|(_, expr)| args::infer_expression_type(&expr))) {
										{#(r#type)}
									} else {
										{#error{ {#root}::concat!("Could not infer type of `", {#root}::stringify!({#(&name)}), "`'s render parameter `", {#root}::stringify!({#(identifier)}), "` from expansion (`as …`) or default (`= …`).") }}
									}}
								}}
								{#if let Some(question) = question { {#located_at(question.question.span()) { > }}}}
								{#(comma)}
							}}
						}

						impl {#root}::Default for {#(&name_render_args_builder_type)} {
							fn default() -> Self {
								{#(&name_render_args_type)}::builder()
							}
						}
					}}
				}}

				impl<
					'r,
					{#if substrate_where.is_some() { {#(substrate)}, }}
				> {#root}::Render<'r, {#(substrate)}> for {#(&name)} {#(substrate_where)} {
					type RenderArgsBuilder = {#(name_render_args_builder_type)};
					type RenderArgs = {#(&name_render_args_type)};
					type RenderContentBuilder = {#(name_render_content_builder_type)};
					type RenderContent = {#(name_render_content_type)};
					type RenderT = <{#(substrate)} as {#root}::Substrate>::RenderTDefault<'r>;
					type RenderESet = <{#(substrate)} as {#root}::Substrate>::RenderESetDefault<'r>;

					fn {#(render)}(
						self: {#root}::Pin<&'r mut Self>,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						args: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderArgs,
						content: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderContent,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						{#root}::ErrorUnion<<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet>,
					> {
						let this = unsafe { &raw mut *{#root}::Pin::into_inner_unchecked(self) };
						{#if let Some(SquareBrackets { span, contents }) = render_args {
							{#located_at(span.join()) {
								let {#(name_render_args_type)} {
									{#for (RenderArg { r#mut, identifier, expansion, .. }, comma) in contents.pairs() {
										{#(r#mut, identifier)}
										{#if let Some((r#as, pattern)) = expansion {
											{#located_at(r#as.span()) {
												:
											}}
											{#(pattern)}
										}}
										{#(comma)}
									}}
								} = args;
							}}
						}}

						let mut acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation({#(body_size_hint)});
						{#(render_proc)}
						<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)
					}
				}

				#[doc(hidden)]
				impl<
					'r,
					{#if substrate_where.is_some() { {#(substrate)}, }}
				> {#root}::ContentParent<'r, {#(substrate)}> for {#(&name)} {#(substrate_where)} {
					type RenderAccumulator = <{#(substrate)} as {#root}::Accumulation<'r>>::RenderAccumulatorDefault;

					#[inline(always)]
					fn start_accumulation(size_hint: {#root}::SizeHint) -> Self::RenderAccumulator {
						<{#(substrate)} as {#root}::Accumulation<'r>>::default_start_accumulation(size_hint)
					}

					#[inline(always)]
					fn finalize_accumulation(
						acc: Self::RenderAccumulator,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						{#root}::ErrorUnion<<Self as {#root}::Render<'r, {#(substrate)}>>::RenderESet>,
					> {
						<{#(substrate)} as {#root}::Accumulation<'r>>::default_finalize_accumulation(acc, ctx)
					}
				}

				#[doc = {#root}::concat!("[`", {#root}::stringify!({#(&name)}), "`] inherits accepted content from [`", {#root}::stringify!({#(substrate)}), "`].")]
				impl<
					'r,
					T,
					{#if substrate_where.is_some() { {#(substrate)}, }}
				> {#root}::ContentParentOf<'r, {#(substrate)}, T> for {#(name)}
				where
					{#(substrate)}: {#root}::SubstrateOnRender<'r, <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator, T>,
					{#(substrate_where.as_ref().map(|sw| &sw.1))}
				{
					#[inline(always)]
					fn on_rendered(
						acc: &'_ mut <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						rendered: T,
					) {
						<{#(substrate)} as {#root}::SubstrateOnRender<'r, <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator, T>>::on_rendered(acc, ctx, rendered)
					}
				}
			}}
		});
	}
}
