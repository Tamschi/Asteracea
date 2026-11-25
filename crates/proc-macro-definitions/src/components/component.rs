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
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use statements::Statement;

use crate::components::component::args::{
	InitArg, InitArgs, InitDep, RenderArg, RenderArgs, SlotArgs,
};

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

		let dep_field_names = if let Some(init_args) = &init_args {
			let mut dep_field_names: Vec<Identifier> = vec![];
			for (
				i,
				(
					InitDep {
						attrs,
						vis,
						r#dyn,
						r#mut,
						pattern,
						question,
						colon,
						r#type,
						default,
					},
					_comma,
				),
			) in init_args.contents.deps.pairs().enumerate()
			{
				if !(r#dyn.is_bracketed() || r#dyn.is_braced()) {
					continue;
				}
				let field_name = Identifier(Ident::new(
					&format!("__Asteracea__dep_{}", fields.len()),
					r#dyn.span().resolved_at(Span::mixed_site()),
				));
				fields.push((attrs.to_owned(), field_name.clone(), colon.clone(), {
					let mut tokens = TokenStream::new();
					quote_into_mixed_site!(r#dyn.span(), root, &mut tokens, {
						{#if question.is_some() && default.is_none() {
							{#root}::Option<
						}}
						<{#(r#type)} as {#root}::Injectable>::Provided
						{#if question.is_some() && default.is_none() {
							>
						}}
					});
					tokens
				}));
				dep_field_names.push(field_name);
			}
			dep_field_names
		} else {
			vec![]
		};

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

		let (name_init_args_type, name_init_args_builder_type) = if let Some(init_args) = &init_args
			&& init_args.contents.args.pairs().next().is_some()
		{
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

			{#if !fields.is_empty() {
				{#mixed_site {
					#[automatically_derived]
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
				}}
			}}

			{#mixed_site {
				{#if let Some(Parentheses { span, contents }) = &init_args
					&& contents.args.pairs().next().is_some() {
					{#located_at(span.join()) {
						#[automatically_derived]
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

						#[automatically_derived]
						impl {#root}::Default for {#(&name_init_args_builder_type)} {
							fn default() -> Self {
								{#(&name_init_args_type)}::builder()
							}
						}
					}}
				}}

				impl {#root}::Tag for {#(&name)} {}

				{#if let Some(r#async) = r#async {
					{#located_at(r#async.span()) {
						#error("TODO: async components")
					}}
				} else {
					// No `#[automatically_derived]` because warnings should be shown here for unused code.
					/// Synchronously initialised.
					impl<
						'i,
						'r,
						{#if substrate_where.is_some() { {#(substrate)}, }}
					> {#root}::Init<'i, 'r, {#(substrate)}> for {#(&name)} {#(substrate_where)} {
						type InitArgsBuilder = {#(name_init_args_builder_type)};
						type InitArgs = {#(&name_init_args_type)};
						type Error = <{#(substrate)} as {#root}::Substrate>::InitErrorDefault<'i>;

						fn {#(init)}(
							deps: &'i mut {#root}::Waypoint<'i, dyn {#root}::Repository>,
							ctx: <{#(substrate)} as {#root}::Substrate>::InitContext<'i>,
							args: <Self as {#root}::Init<'i, 'r, {#(substrate)}>>::InitArgs,
						) -> impl 'i + {#root}::PinInit<
							Self,
							<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::Error,
						> {#located_at(fat_arrow.eq.span().join(fat_arrow.gt.span()).unwrap_or_else(|| fat_arrow.eq.span())){
							{
								let init = move |this: *mut Self| -> {#root}::core::result::Result<
									(),
									<Self as {#root}::Init<'i, 'r, {#(substrate)}>>::Error,
								> {
									{#if let Some(Parentheses{span, contents}) = &init_args {
										{#if contents.args.pairs().next().is_some() {
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

										{#let mut dep_field_names = dep_field_names.iter();}
										{#for (InitDep { attrs, vis, r#dyn, r#mut, pattern, question, colon, r#type, default }, comma) in contents.deps.pairs() {
											{#located_at(r#dyn.span()) {
												{#if r#dyn.is_bracketed() || r#dyn.is_braced() {
													{#let Some(field_name) = dep_field_names.next() else { unreachable!() };}
													{#(attrs)}
													let {#(&field_name)} = {
														let value = deps.extract::<{#(r#type)}>()
															{#if let Some(question) = question {
																{#located_at(question.question.span()) {
																	.ok()
																}}
																{#if let Some((eq, expr)) = default {
																	{#located_at(eq.eq.span()) {
																		.unwrap_or_else(|| {#(expr)})
																	}}
																}}
															} else {
																{#if let Some((eq, expr)) = default {
																	{#located_at(eq.eq.span()) {
																		.unwrap_or_else(|_| {#(expr)})
																	}}
																} else if question.is_none() {
																	?
																}}
															}};
														unsafe { {#root}::pinned_init_and_guard(value, &raw mut (*this).{#(&field_name)})? }
													};
													{#if r#dyn.is_braced() {
														let {#(pattern)} = unsafe {
															let ptr = *{#(&field_name)};
															{#root}::{#if r#mut.is_some() { co_borrow_mut } else { co_borrow }}(
																&{#(r#mut, &field_name)},
																ptr,
															)
														};
													}}
												} else {
													{#(attrs)}
													let {#(r#mut, pattern)} = deps.extract::<{#(r#type)}>()
														{#if let Some(question) = question {
															{#located_at(question.question.span()) {
																.ok()
															}}
															{#if let Some((eq, expr)) = default {
																{#located_at(eq.eq.span()) {
																	.unwrap_or_else(|| {#(expr)})
																}}
															}}
														} else {
															{#if let Some((eq, expr)) = default {
																{#located_at(eq.eq.span()) {
																	.unwrap_or_else(|_| {#(expr)})
																}}
															} else if question.is_none() {
																?
															}}
														}}
														;
												}}
											}}
										}}
										{#let _ = assert_eq!(dep_field_names.count(), 0);}

										let deps = &mut deps.new::<Self, _>((
											//TODO: Provide injectables.
										));
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
					type Error = <{#(substrate)} as {#root}::Substrate>::RenderErrorDefault<'r>;

					fn {#(render)}(
						self: {#root}::Pin<&'r mut Self>,
						ctx: <{#(substrate)} as {#root}::Substrate>::RenderContext<'r>,
						args: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderArgs,
						content: <Self as {#root}::Render<'r, {#(substrate)}>>::RenderContent,
					) -> {#root}::Result<
						<Self as {#root}::Render<'r, {#(substrate)}>>::RenderT,
						<Self as {#root}::Render<'r, {#(substrate)}>>::Error,
					> {
						let this = unsafe { &raw mut *{#root}::Pin::into_inner_unchecked(self) };

						{#if let Some(Parentheses { span: _ , contents }) = init_args {
							{#let mut dep_field_names = dep_field_names.into_iter();}
							{#for (InitDep { attrs, vis, r#dyn, r#mut, pattern, question, colon, r#type, default }, comma) in contents.deps.pairs() {
								{#if r#dyn.is_bracketed() || r#dyn.is_braced() {
									{#located_at(r#dyn.span()) {
										{#let Some(field_name) = dep_field_names.next() else { unreachable!() };}
										{#(attrs)}
										let {#(&field_name)} = {#root}::GuardToken;
										let {#(pattern)} = &{#(r#mut)} **unsafe {
											let ptr = &raw {#if let Some(r#mut) = r#mut { {#(r#mut)} } else { const }} (*this).{#(&field_name)};
											{#root}::{#if r#mut.is_some() { co_borrow_mut } else { co_borrow }}(
												&{#(&field_name)},
												ptr,
											)
										};
									}}
								}}
							}}
							{#let _ = assert_eq!(dep_field_names.count(), 0);}
						}}

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
						<Self as {#root}::Render<'r, {#(substrate)}>>::Error,
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
					) -> {#root}::Result<(), <Self as {#root}::Render<'r, {#(substrate)}>>::Error> {
						<{#(substrate)} as {#root}::SubstrateOnRender<'r, <Self as {#root}::ContentParent<'r, {#(substrate)}>>::RenderAccumulator, T>>::on_rendered(acc, ctx, rendered)
					}
				}
			}}
		});
	}
}
