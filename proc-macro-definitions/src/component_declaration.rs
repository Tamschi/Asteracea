use self::{
	arguments::{Argument, ConstructorArgument},
	parameter_helper_definitions::{CustomArgument, ParameterHelperDefinitions},
};
use crate::{
	asteracea_ident,
	part::{GenerateContext, LetSelf},
	storage_configuration::StorageTypeConfiguration,
	storage_context::{ParseContext, ParseWithContext, StorageContext},
	syn_ext::{AddOptionExt, *},
	util::Braced,
	warn, Configuration, MapMessage, Part,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::DebuglessUnwrap as _;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use std::rc::Rc;
use syn::{
	parenthesized,
	parse::{discouraged, Parse, ParseStream, Result},
	parse2, parse_quote_spanned,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Paren,
	AttrStyle, Attribute, Error, FieldPat, Generics, Ident, Item, Lifetime, Member, Pat, PatIdent,
	PatType, ReturnType, Token, Type, Visibility, WhereClause, WherePredicate,
};
use tap::Pipe as _;
use unquote::unquote;

mod arguments;
mod parameter_helper_definitions;
mod kw {
	syn::custom_keyword!(new);
	syn::custom_keyword!(Sync);
	syn::custom_keyword!(with);
}

pub struct ComponentDeclaration {
	attributes: Vec<Attribute>,
	visibility: Visibility,
	async_: Option<Token![async]>,
	name: Ident,
	component_generics: Generics,
	storage_context: StorageContext,
	constructor_attributes: Vec<Attribute>,
	constructor_generics: Generics,
	constructor_paren: Paren,
	constructor_args: Punctuated<ConstructorArgument, Token![,]>,
	render_attributes: Vec<Attribute>,
	render_generics: Generics,
	render_paren: Paren,
	render_args: Punctuated<Argument, Token![,]>,
	render_type: RenderType,
	constructor_block: Option<(kw::new, kw::with, Braced)>,
	body: Part<ComponentRenderConfiguration>,
	assorted_items: Vec<Item>,
	callback_registrations: Vec<(Ident, Type)>,
}

pub enum RenderType {
	AutoSafe,
	Explicit(Token![->], Box<Type>),
	ExplicitAutoSync(Token![->], kw::Sync, Token![?]),
	Sync(Token![->], kw::Sync),
	UnSync(Token![->], Token![!], kw::Sync),
}

pub struct FieldDefinition {
	pub attributes: Vec<Attribute>,
	pub visibility: Visibility,
	pub name: Ident,
	pub field_type: Type,
	pub initial_value: TokenStream,
	pub structurally_pinned: bool,
}

enum ComponentRenderConfiguration {}
impl Configuration for ComponentRenderConfiguration {
	const NAME: &'static str = "component! render expression";
	const CAN_CAPTURE: bool = true;
}

impl Parse for ComponentDeclaration {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let attributes = input.call(Attribute::parse_outer)?;
		let visibility = input.parse()?;
		let async_ = input.parse().expect("infallible");

		let component_name: Ident = input
			.parse()
			.map_message("Expected identifier (component name).")?;

		let mut component_generics = if input.peek(Token![<]) {
			input.parse()?
		} else {
			Generics::default()
		};

		if input.peek(Token![where]) {
			let where_token = input.parse::<Token![where]>()?;
			let mut component_wheres: Vec<WherePredicate> = Vec::new();
			loop {
				let forked = input.fork();
				if let Ok(predicate) = forked.parse() {
					let comma_required = !forked.peek(Token![#]);
					if comma_required && forked.parse::<Token![,]>().is_err() {
						return Err(Error::new_spanned(
							predicate,
							"A component's where clause must end with a comma unless it is followed by an attribute.",
						));
					}
					component_wheres.push(predicate);

					// Not great, but it's still the cleanest solution I think.
					// No generics are valid where predicates, since they'd have to be followed by :.
					discouraged::Speculative::advance_to(input, &forked);
					if !comma_required {
						break;
					}
				} else {
					break;
				}
			}
			if component_wheres.is_empty() {
				warn(
					input.cursor().span(),
					"No where predicate found.
					Did you forget to end it with a comma?",
				)?;
			} else {
				component_generics.where_clause = component_generics
					.where_clause
					.add(&Some(WhereClause {
						where_token,
						predicates: component_wheres.into_iter().collect(),
					}))
					.as_deref()
					.cloned()
			}
		}
		let component_generics = component_generics;

		let constructor_attributes = input.call(Attribute::parse_outer)?;

		//TODO: Where clause, somehow.
		let constructor_generics = if input.peek(Token![<]) {
			input.parse()?
		} else {
			Generics::default()
		};

		if !input.peek(Paren) {
			let message = "Expected parentheses (constructor arguments).".to_string();
			return Err(Error::new(
				input.cursor().span(),
				if matches!(component_generics.where_clause, Some(where_clause) if !where_clause.predicates.is_empty())
				{
					message + "\nDid you forget to end the component where clause with a comma?"
				} else {
					message
				},
			));
		}
		let constructor_args;
		let constructor_paren = parenthesized!(constructor_args in input); //TODO: Specify error message.
		let constructor_args = Punctuated::parse_terminated(&constructor_args)?;

		let render_attributes = input.call(Attribute::parse_outer)?;

		//TODO: Where clause, somehow.
		let render_generics = if input.peek(Token![<]) {
			input.parse()?
		} else {
			Generics::default()
		};

		let render_args;
		let render_paren = parenthesized!(render_args in input); //TODO: Specify error message.
		let render_args = Punctuated::parse_terminated(&render_args)?;

		let render_type = input.parse()?;

		let mut cx = ParseContext::new_root(&visibility, &component_name, &component_generics);

		let constructor_block = if input.peek(kw::new) {
			unquote! {input,
				#let new
				#let with
				#let block
			};
			Some((new, with, block))
		} else {
			None
		};

		let body = loop {
			match Part::parse_with_context(input, &mut cx)? {
				None => (),
				Some(body) => break body,
			}
		};

		// These captures are put at the very end of the constructor since they always move their value.
		for constructor_argument in constructor_args.iter() {
			if let ConstructorArgument {
				capture: arguments::Capture::Yes(visibility),
				injection_dyn,
				argument: Argument {
					fn_arg, question, ..
				},
			} = constructor_argument
			{
				let span = match visibility {
					Visibility::Inherited => fn_arg.span(),
					visibility => visibility.span(),
				};
				let attrs = &fn_arg.attrs;
				let pat = &fn_arg.pat;
				let arg = {
					let PatType {
						colon_token, ty, ..
					} = fn_arg;

					let mut ty = match injection_dyn {
						None => Type::clone(ty),
						Some(dyn_) => {
							let asteracea = asteracea_ident(dyn_.span);
							parse_quote_spanned! {dyn_.span.resolved_at(Span::mixed_site())=>
								<
									<#ty as ::#asteracea::__::rhizome::sync::Extract>::Extracted
									as ::#asteracea::__::rhizome::sync::Extracted<::core::any::TypeId>
								>::Extracted
							}
						}
					};

					if let Some(question) = question {
						ty = parse_quote_spanned! {question.span.resolved_at(Span::mixed_site())=>
							::core::option::Option::<#ty>
						}
					}

					quote!(#pat #colon_token #ty)
				};

				let attrs = attrs
					.iter()
					.map(|attr| Attribute {
						style: AttrStyle::Inner(Token![!](
							attr.pound_token.span.resolved_at(Span::mixed_site()),
						)),
						..attr.clone()
					})
					.collect::<Vec<_>>();

				call2_strict(
					quote_spanned!(span=> let #visibility self.#arg = #(#attrs)* #pat;),
					|input| {
						LetSelf::<ComponentRenderConfiguration>::parse_with_context(input, &mut cx)
					},
				)
				.debugless_unwrap()?;
			}
		}

		if !input.is_empty() {
			return Err(input.error(
				"Currently, only one root element is supported.
				Consider wrapping your elements like so: <div child1 child2 ...>",
			));
		}

		let ParseContext {
			assorted_items,
			storage_context,
			callback_registrations,
			..
		} = cx;

		Ok(Self {
			assorted_items,
			attributes,
			storage_context,
			visibility,
			async_,
			name: component_name,
			component_generics,
			constructor_attributes,
			constructor_generics,
			constructor_paren,
			constructor_args,
			render_attributes,
			render_generics,
			render_paren,
			render_args,
			render_type,
			constructor_block,
			body,
			callback_registrations: Rc::try_unwrap(callback_registrations)
				.expect(
					"Internal Asteracea error: `callback_registrations` still referenced elsewhere",
				)
				.into_inner(),
		})
	}
}

impl Parse for RenderType {
	fn parse(input: ParseStream) -> Result<Self> {
		match input.parse().unwrap() {
			None => Self::AutoSafe,
			Some(r_arrow) => match input.parse().unwrap() {
				Some(bang) => Self::UnSync(r_arrow, bang, input.parse()?),
				None => match input.parse().unwrap() {
					Some(sync) => match input.parse().unwrap() {
						Some(question) => Self::ExplicitAutoSync(r_arrow, sync, question),
						None => Self::Sync(r_arrow, sync),
					},
					None => Self::Explicit(r_arrow, input.parse()?),
				},
			},
		}
		.pipe(Ok)
	}
}

impl ComponentDeclaration {
	#[allow(clippy::cognitive_complexity)]
	pub fn into_tokens(self) -> Result<TokenStream> {
		let Self {
			attributes,
			visibility,
			async_,
			name: component_name,
			mut storage_context,
			component_generics,
			constructor_attributes,
			constructor_generics,
			constructor_paren,
			constructor_args,
			render_attributes,
			render_generics,
			render_paren,
			render_args,
			render_type,
			constructor_block,
			body,
			assorted_items: mut random_items,
			callback_registrations,
		} = self;

		let asteracea = asteracea_ident(Span::call_site());

		let mut unsafe_drop_early = TokenStream::new();
		for (name, parameter_type) in callback_registrations {
			storage_context.push(FieldDefinition {
				attributes: vec![],
				visibility: Visibility::Inherited,
				name: name.clone(),
				field_type: parse_quote_spanned! {parameter_type.span().resolved_at(Span::mixed_site())=>
					::#asteracea::__::DroppableLazyCallbackRegistration::<
						#component_name,
						fn(#parameter_type),
					>
				},
				initial_value: quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
					::#asteracea::__::DroppableLazyCallbackRegistration::default()
				},
				structurally_pinned: true, // This isn't quite clean, but it implies asserting `!Unpin` on the component type.
			});

			// IMPORTANT: These fields must be dropped FIRST, before any other drop logic!
			// Dropping callback registrations synchronises callbacks to make sure they aren't newly invoked,
			// and that all running ones have exited already.

			// Dropping these callback registrations early also means they mustn't be considered user-accessible.
			assert!(name.to_string().contains("__Asteracea__"));
			unsafe_drop_early.extend(
				quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
					::#asteracea::__::DroppableLazyCallbackRegistration::drop(&mut self.#name);
				},
			)
		}

		let struct_definition = StorageTypeConfiguration::new_component_root(
			component_name.clone(),
			component_generics.clone(),
		)
		.struct_definition(
			attributes.to_vec(),
			visibility.clone(),
			component_name.clone(),
			&storage_context,
			&Generics::default(),
		)?;

		let constructed_value = storage_context.value(
			true,
			&parse2(component_name.to_token_stream()).unwrap(),
			false,
		);

		let bump = quote_spanned! (render_paren.span.join().resolved_at(Span::call_site())=>
			bump
		);

		let cx = match render_type {
			RenderType::AutoSafe => GenerateContext {
				thread_safety: quote!(_),
				prefer_thread_safe: Some(quote!(.prefer_thread_safe())),
			},
			RenderType::Explicit(r_arrow, _) => GenerateContext {
				thread_safety: quote_spanned!(r_arrow.span()=> _),
				prefer_thread_safe: None,
			},
			RenderType::ExplicitAutoSync(r_arrow, _, sync) => GenerateContext {
				thread_safety: quote_spanned!(sync.span.resolved_at(Span::mixed_site())=> _),
				prefer_thread_safe: Some(quote_spanned!(r_arrow.span()=> .prefer_thread_safe())),
			},
			RenderType::Sync(_, sync) => GenerateContext {
				thread_safety: quote_spanned!(sync.span.resolved_at(Span::mixed_site())=> ::#asteracea::lignin::ThreadSafe),
				prefer_thread_safe: None,
			},
			RenderType::UnSync(_, _, sync) => GenerateContext {
				thread_safety: quote_spanned!(sync.span.resolved_at(Span::mixed_site())=> ::#asteracea::lignin::ThreadBound),
				prefer_thread_safe: None,
			},
		};

		let body = body.part_tokens(&cx)?;

		let new_lifetime: Lifetime = parse2(quote_spanned!(Span::call_site()=> 'NEW)).unwrap();
		let render_lifetime: Lifetime =
			parse2(quote_spanned!(Span::call_site()=> 'RENDER)).unwrap();

		let (injected_args, constructor_args): (Vec<_>, Vec<_>) = constructor_args
			.into_iter()
			.partition(|arg| arg.injection_dyn.is_some());

		let mut injected_pats = Vec::<&Pat>::new();
		let mut dependency_extractions = Vec::<TokenStream>::new();
		for injected_arg in &injected_args {
			injected_pats.push(&*injected_arg.argument.fn_arg.pat);

			let span = injected_arg
				.injection_dyn
				.as_ref()
				.unwrap()
				.span
				.resolved_at(Span::mixed_site());
			let ty = &*injected_arg.argument.fn_arg.ty;
			let value = quote_spanned! {span=>
				<#ty as ::#asteracea::__::rhizome::sync::Extract>::extract(parent_node)
					.map_err(::#asteracea::error::IncompatibleRuntimeDependency::<#ty>::new_and_log)
					.map_err(::#asteracea::error::Escalate::escalate)?
			};
			let value = match (
				injected_arg.argument.question.as_ref(),
				injected_arg.argument.default.as_ref(),
			) {
				(None, None) => quote_spanned! {span=>
					#value
						.ok_or_else(|| ::#asteracea::error::RuntimeDependencyMissing::<#ty>::new_and_log())
						.map_err(::#asteracea::error::Escalate::escalate)?
				},
				(None, Some((eq, default))) => {
					quote_spanned! {eq.span.resolved_at(Span::mixed_site())=>
						#value.unwrap_or_else(|| #default)
					}
				}
				(Some(_), None) => value,
				(Some(_), Some(_)) => todo!("explicitly optional dependency with default"),
			};
			dependency_extractions.push(value);
		}

		let custom_new_args = constructor_args
			.iter()
			.map(|arg| Ok(CustomArgument {
				attrs: arg.argument.fn_arg.attrs.as_slice(),
				ident: match &*arg.argument.fn_arg.pat {
					Pat::Ident(PatIdent{ ident, .. }) => ident,
					other => {return Err(Error::new_spanned(other, "Component parameters must be named. Bind this pattern to an identifier by prefixing it with `identifier @`."))}
				},
				optional: arg.argument.question,
				ty: &*arg.argument.fn_arg.ty,
				default: &arg.argument.default,
			}))
			.collect::<Result<Vec<_>>>()?;

		let custom_render_args = render_args
			.iter()
			.map(|arg| Ok(CustomArgument {
				attrs: arg.fn_arg.attrs.as_slice(),
				ident: match &*arg.fn_arg.pat {
					Pat::Ident(PatIdent{ ident, .. }) => ident,
					other => {return Err(Error::new_spanned(other, "Component parameters must be named. Bind this pattern to an identifier by prefixing it with `identifier @`."))}
				},
				optional: arg.question,
				ty: &*arg.fn_arg.ty,
				default: &arg.default,
			}))
			.collect::<Result<Vec<_>>>()?;

		let ParameterHelperDefinitions {
			on_parameter_struct: new_args_generics,
			parameter_struct_body: new_args_body,
			on_function: new_generics,
			for_function_args: new_args_generic_args,
			on_builder_function: new_args_builder_generics,
			for_builder_function_return: new_args_builder_generic_args,
		} = ParameterHelperDefinitions::new(
			&component_generics,
			&parse2(quote_spanned!(constructor_paren.span=> <'a: '_>)).unwrap(),
			&constructor_generics,
			custom_new_args.as_slice(),
			&new_lifetime,
		);

		let ParameterHelperDefinitions {
			on_parameter_struct: render_args_generics,
			parameter_struct_body: render_args_body,
			on_function: render_generics,
			for_function_args: render_args_generic_args,
			on_builder_function: render_args_builder_generics,
			for_builder_function_return: render_args_builder_generic_args,
		} = ParameterHelperDefinitions::new(
			&component_generics,
			&parse2(quote_spanned!(render_paren.span=> <'a, 'bump: '_>)).unwrap(),
			&render_generics,
			custom_render_args.as_slice(),
			&render_lifetime,
		);

		let constructor_args_field_patterns = constructor_args
			.into_iter()
			.map(|arg| match *arg.argument.fn_arg.pat {
				Pat::Ident(pat_ident) => pat_ident.try_into_field_pat(),
				_ => {
					unreachable!()
				}
			})
			.collect::<Result<Vec<_>>>()?;

		let constructor_args_tracing_fields = constructor_args_field_patterns
			.iter()
			.map(|FieldPat { member, .. }| match member {
				Member::Unnamed(_) => unreachable!(),
				Member::Named(arg_ident) => {
					quote_spanned!(arg_ident.span().resolved_at(Span::mixed_site())=> #arg_ident = {
						use ::#asteracea::__::CoerceTracingValue;
						(&&&&&::#asteracea::__::InertWrapper(&args.#arg_ident)).coerce()
					})
				}
			})
			.collect::<Vec<_>>();

		let render_args_field_patterns = render_args
			.into_iter()
			.map(|arg| match *arg.fn_arg.pat {
				Pat::Ident(pat_ident) => pat_ident.try_into_field_pat(),
				_ => {
					unreachable!()
				}
			})
			.collect::<Result<Vec<_>>>()?;

		let render_args_tracing_fields = render_args_field_patterns
			.iter()
			.map(|FieldPat { member, .. }| match member {
				Member::Unnamed(_) => unreachable!(),
				Member::Named(arg_ident) => {
					quote_spanned!(arg_ident.span().resolved_at(Span::mixed_site())=> #arg_ident = {
						use ::#asteracea::__::CoerceTracingValue;
						(&&&&&::#asteracea::__::InertWrapper(&args.#arg_ident)).coerce()
					})
				}
			})
			.collect::<Vec<_>>();

		// These can't be fully hygienic with current technology.
		let new_args_name =
			Ident::new(&format!("{}NewArgs", component_name), component_name.span());
		let render_args_name = Ident::new(
			&format!("{}RenderArgs", component_name),
			component_name.span(),
		);

		let new_args_builder_name =
			Ident::new(&format!("{}Builder", new_args_name), component_name.span());
		let render_args_builder_name = Ident::new(
			&format!("{}Builder", render_args_name),
			component_name.span(),
		);

		let render_self: Token![self] = parse2(quote_spanned!(render_paren.span=> self)).unwrap();

		let render_type: ReturnType = match render_type {
			RenderType::AutoSafe => {
				let auto_safe = Ident::new(
					(component_name.to_string() + "__Asteracea__AutoSafe")
						.trim_start_matches("r#"),
					Span::mixed_site(),
				);
				random_items.push(
					parse2(quote! {
						::#asteracea::lignin::auto_safety::AutoSafe_alias!(pub(crate) #auto_safe);
					})
					.expect("RenderType::AutoSafe __Asteracea__AutoSafe"),
				);
				parse2(quote! {
					-> ::std::result::Result<
						impl #auto_safe<::#asteracea::lignin::Node<'bump, ::#asteracea::lignin::ThreadBound>>,
						::#asteracea::error::Escalation,
					>
				})
				.expect("render_type AutoSafe")
			}
			RenderType::Explicit(r_arrow, type_) => ReturnType::Type(
				r_arrow,
				parse2(quote_spanned! {r_arrow.span()=>
					::std::result::Result<#type_, ::#asteracea::error::Escalation>
				})
				.expect("RenderType::Explicit"),
			),
			RenderType::ExplicitAutoSync(_, _, question) => {
				parse2(quote_spanned! {question.span=>
					-> ::std::result::Result<
						impl ::#asteracea::lignin::auto_safety::AutoSafe::<::#asteracea::lignin::Node<'bump, ::#asteracea::lignin::ThreadBound>>,
						::#asteracea::error::Escalation,
					>
				})
				.expect("render_type AutoSafe")
			}
			RenderType::Sync(r_arrow, _) | RenderType::UnSync(r_arrow, _, _) => {
				let thread_safety = &cx.thread_safety;
				parse2(quote_spanned! {r_arrow.span()=>
					-> ::std::result::Result<::#asteracea::lignin::Node<'bump, #thread_safety>, ::#asteracea::error::Escalation>
				})
				.expect("render_type explicit thread safety")
			}
		};

		let constructor_block_statements =
			constructor_block.map(|(_new, _with, block)| block.contents);

		let call_site_node = Ident::new("node", Span::call_site());

		let (component_impl_generics, component_type_generics, component_where_clause) =
			component_generics.split_for_impl();

		// This (hopefully) enables unused function warnings.
		let new = Ident::new("new", component_name.span());
		let render = Ident::new("render", component_name.span());

		let mut new_span_name = Literal::string(&format!("{}::{}", component_name, new));
		new_span_name.set_span(new.span().resolved_at(Span::mixed_site()));
		let mut render_span_name = Literal::string(&format!("{}::{}", component_name, render));
		render_span_name.set_span(render.span().resolved_at(Span::mixed_site()));

		//FIXME:
		//  This can be solved for async components using <https://github.com/tokio-rs/tracing/pull/1819> once that lands.
		//  However, that macro is currently unhygienic (which causes e.g. the parameter names `debug` and `display` to collide), so that issue must be solved too.
		let constructor_tracing_span = if async_.is_none() {
			Some(quote_spanned! {Span::mixed_site()=>
				// Tracing's `#[instrument]` macro is slightly unwieldy in terms of compilation.
				// The following should be equivalent to skipping all fields and setting them one by one:
				let _tracing_span = ::#asteracea::__::tracing::debug_span!(#new_span_name, #(#constructor_args_tracing_fields,)*).entered();
			})
		} else {
			None
		};

		Ok(quote_spanned! {Span::mixed_site()=>
			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::__::typed_builder::TypedBuilder)]
			#[builder(doc)]
			#[allow(non_snake_case)]
			#visibility struct #new_args_name #new_args_generics #new_args_body

			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::__::typed_builder::TypedBuilder)]
			#[builder(doc)]
			#[allow(non_snake_case)]
			#visibility struct #render_args_name #render_args_generics #render_args_body

			#(#struct_definition)*

			impl #component_impl_generics #component_name #component_type_generics #component_where_clause {
				/// <!-- (suppress `missing_docs`) -->
				#(#constructor_attributes)*
				pub #async_ fn #new #new_generics(
					parent_node: ::core::pin::Pin<&::#asteracea::__::rhizome::sync::Node<
						::core::any::TypeId,
						::core::any::TypeId,
						::#asteracea::__::rhizome::sync::DynValue,
					>>,
					args: #new_args_name #new_args_generic_args,
				) -> ::std::result::Result<Self, ::#asteracea::error::Escalation> where Self: 'a + 'static { // TODO: Self: 'static is necessary because of `derive_for::<Self>`, but that's not really a good approach... Using derived IDs would be better.
					#constructor_tracing_span

					// These are assigned at once to make sure name collisions error.
					let (#new_args_name {
						#(#constructor_args_field_patterns,)*
						__Asteracea__phantom: _,
					}, (#(#injected_pats,)*)) = (args, (#(#dependency_extractions,)*));

					let mut #call_site_node = parent_node.branch_for(::core::any::TypeId::of::<Self>());

					{} // Isolate constructor block.
					#constructor_block_statements
					{} // Dito.

					::std::result::Result::Ok(#constructed_value)
				}

				/// <!-- (suppress `missing_docs`) -->
				pub fn new_args_builder #new_args_builder_generics()
				-> #new_args_builder_name #new_args_builder_generic_args {
					#new_args_name::builder()
				}

				/// <!-- (suppress `missing_docs`) -->
				#(#render_attributes)*
				pub fn #render #render_generics(
					#render_self: ::std::pin::Pin<&'a Self>,
					#bump: &'bump #asteracea::bumpalo::Bump,
					args: #render_args_name #render_args_generic_args,
				) #render_type {
					// Tracing's `#[instrument]` macro is slightly unwieldy in terms of compilation.
					// The following should be equivalent to skipping all fields and setting them one by one:
					let _tracing_span = ::#asteracea::__::tracing::debug_span!(#render_span_name, #(#render_args_tracing_fields,)*).entered();

					let #render_args_name {
						#(#render_args_field_patterns,)*
						__Asteracea__phantom: _,
					} = args;

					let this = #render_self;
					::std::result::Result::Ok(#body)
				}

				/// <!-- (suppress `missing_docs`) -->
				pub fn render_args_builder #render_args_builder_generics()
				-> #render_args_builder_name #render_args_builder_generic_args {
					#render_args_name::builder()
				}

				//TODO: Is it possible to call render_args_builder on the reference instead, somehow?
				#[doc(hidden)] // This is used for inference in generated code.
				#[allow(non_snake_case)]
				pub fn __Asteracea__ref_render_args_builder #render_args_builder_generics(&self)
				-> #render_args_builder_name #render_args_builder_generic_args {
					let _ = self;
					#render_args_name::builder()
				}
			}

			#(#random_items)*

			/// Asteracea components do not currently support custom [`Drop`](`::std::ops::Drop`) implementations.
			impl #component_impl_generics ::std::ops::Drop for #component_name #component_type_generics #component_where_clause {
				fn drop(&mut self) {
					unsafe {
						#unsafe_drop_early
					}
					//TODO: Undo DOM bindings.
				}
			}
		})
	}
}
