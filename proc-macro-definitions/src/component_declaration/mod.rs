use self::{
	arguments::{ConstructorArgument, ValidatedArgument},
	parameter_helper_definitions::{CustomArgument, ParameterHelperDefintions},
};
use crate::{
	asteracea_ident,
	part::GenerateContext,
	storage_configuration::StorageTypeConfiguration,
	storage_context::{ParseContext, ParseWithContext, StorageContext},
	syn_ext::{AddOptionExt, *},
	warn, Configuration, MapMessage, Part,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap as _, DebuglessUnwrapNone as _};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	parenthesized,
	parse::{discouraged, Parse, ParseStream, Result},
	parse2,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Paren,
	Attribute, Error, Generics, Ident, Item, Lifetime, Pat, PatIdent, ReturnType, Token, Type,
	Visibility, WhereClause, WherePredicate,
};
use syn_mid::Block;
use unquote::unquote;

mod arguments;
mod parameter_helper_definitions;
mod kw {
	syn::custom_keyword!(new);
	syn::custom_keyword!(with);
}

pub struct ComponentDeclaration {
	attributes: Vec<Attribute>,
	visibility: Visibility,
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
	render_args: Punctuated<ValidatedArgument, Token![,]>,
	render_type: ReturnType,
	constructor_block: Option<(kw::new, kw::with, Block)>,
	body: Part<ComponentRenderConfiguration>,
	rhizome_extractions: Vec<TokenStream>,
	assorted_items: Vec<Item>,
}

pub struct FieldDefinition {
	pub attributes: Vec<Attribute>,
	pub visibility: Visibility,
	pub name: Ident,
	pub field_type: TokenStream,
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

		let mut rhizome_extractions = Vec::new();

		let mut cx = ParseContext::new_root(&visibility, &component_name, &component_generics);

		// Dependency extraction:
		while let Some(ref_token) = input.parse::<Token![ref]>().ok() {
			let rhizome_lookahead = input.lookahead1();
			if rhizome_lookahead.peek(Token![;]) {
				input.parse::<Token![;]>()?;
				//TODO: Warn if this is unnecessary!
				continue;
			} else if rhizome_lookahead.peek(Token![for]) {
				let extracted_for = input.parse::<Token![for]>()?;
				let scope: Lifetime = input.parse()?;
				if scope.ident != "NEW" {
					return Err(Error::new_spanned(scope, "Expected 'NEW."));
				}

				let extracted_let = quote_spanned!(ref_token.span=> let);
				let extracted_name: Ident = input.parse()?;
				let extracted_colon: Token![:] = input.parse()?;
				let extracted_type: Type = input.parse()?;
				let extracted_question: Token![?] = input.parse()?;

				let semi = Token![;](extracted_question.span);

				let asteracea = asteracea_ident(extracted_for.span);

				//TODO: Is there a way to write this span more nicely?
				let ref_statement_span = quote!(#ref_token #extracted_for #scope #extracted_name #extracted_colon #extracted_type).span();
				let call_site_node =
					Ident::new("node", ref_statement_span.resolved_at(Span::call_site()));
				rhizome_extractions.push({
					quote_spanned! {
						ref_statement_span=>
						#extracted_let #extracted_name#extracted_colon std::sync::Arc<#extracted_type>
						= <#extracted_type>::extract_from(&#call_site_node)
							.map_err(|error| ::#asteracea::error::Escalate2::escalate(::std::format!("Dependency resolution error in component {}: {}", ::std::stringify!(#component_name), error)))#extracted_question#semi
					}
				})
			} else {
				return Err(rhizome_lookahead.error());
			}
		}

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
				argument,
			} = constructor_argument
			{
				let fn_arg = &argument.fn_arg;
				let span = match visibility {
					Visibility::Inherited => fn_arg.span(),
					visibility => visibility.span(),
				};
				let attrs = &fn_arg.attrs;
				let pat = &fn_arg.pat;
				let arg = {
					let colon_token = fn_arg.colon_token;
					let ty = argument.effective_type();
					quote!(#pat#colon_token #ty)
				};
				call2_strict(
					quote_spanned!(span=> |#(#attrs)* #visibility #arg = {#pat}|;),
					|input| {
						Part::<ComponentRenderConfiguration>::parse_with_context(input, &mut cx)
					},
				)
				.debugless_unwrap()?
				.debugless_unwrap_none()
			}
		}

		if !input.is_empty() {
			return Err(input.error(
				"Currently, only one root element is supported.
                Consider wrapping your elements like so: <div child1 child2 ...>",
			));
		}

		Ok(Self {
			assorted_items: cx.assorted_items,
			attributes,
			storage_context: cx.storage_context,
			visibility,
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
			rhizome_extractions,
		})
	}
}

impl ComponentDeclaration {
	#[allow(clippy::cognitive_complexity)]
	pub fn into_tokens(self) -> Result<TokenStream> {
		let Self {
			attributes,
			visibility,
			name: component_name,
			storage_context,
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
			rhizome_extractions,
			assorted_items: random_items,
		} = self;

		let asteracea = asteracea_ident(Span::call_site());

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

		let bump = quote_spanned! (render_paren.span.resolved_at(Span::call_site())=>
			bump
		);

		let body = body.part_tokens(&GenerateContext::default())?;

		let new_lifetime: Lifetime = parse2(quote_spanned!(Span::call_site()=> 'NEW)).unwrap();
		let render_lifetime: Lifetime =
			parse2(quote_spanned!(Span::call_site()=> 'RENDER)).unwrap();

		let custom_new_args = constructor_args
			.iter()
			.map(|arg| Ok(CustomArgument {
				attrs: arg.argument.fn_arg.attrs.as_slice(),
				item_name: &arg.argument.item_name,
				ident: match &*arg.argument.fn_arg.pat {
				    Pat::Ident(PatIdent{ ident, .. }) => ident,
				    other => {return Err(Error::new_spanned(other, "Component parameters must be named. Bind this pattern to an identifier by prefixing it with `identifier @`."))}
				},
				repeat_mode: arg.argument.repeat_mode,
				optional: arg.argument.optional,
				flatten: &arg.argument.flatten,
				ty: &*arg.argument.fn_arg.ty,
				default: &arg.argument.default,
			}))
			.collect::<Result<Vec<_>>>()?;

		let custom_render_args = render_args
			.iter()
			.map(|arg| Ok(CustomArgument {
				attrs: arg.fn_arg.attrs.as_slice(),
				item_name: &arg.item_name,
				ident: match &*arg.fn_arg.pat {
				    Pat::Ident(PatIdent{ ident, .. }) => ident,
				    other => {return Err(Error::new_spanned(other, "Component parameters must be named. Bind this pattern to an identifier by prefixing it with `identifier @`."))}
				},
				repeat_mode: arg.repeat_mode,
				optional: arg.optional,
				flatten: &arg.flatten,
				ty: &*arg.fn_arg.ty,
				default: &arg.default,
			}))
			.collect::<Result<Vec<_>>>()?;

		let ParameterHelperDefintions {
			on_parameter_struct: new_args_generics,
			parameter_struct_body: new_args_body,
			on_function: new_generics,
			for_function_args: new_args_generic_args,
			on_builder_function: new_args_builder_generics,
			for_builder_function_return: new_args_builder_generic_args,
			has_impl_generics: _new_has_impl_generics,
		} = ParameterHelperDefintions::new(
			&component_generics,
			&parse2(quote_spanned!(constructor_paren.span=> <'a: '_>)).unwrap(),
			&constructor_generics,
			custom_new_args.as_slice(),
			&new_lifetime,
		);

		let ParameterHelperDefintions {
			on_parameter_struct: render_args_generics,
			parameter_struct_body: render_args_body,
			on_function: render_generics,
			for_function_args: render_args_generic_args,
			on_builder_function: render_args_builder_generics,
			for_builder_function_return: render_args_builder_generic_args,
			has_impl_generics: _render_has_impl_generics,
		} = ParameterHelperDefintions::new(
			&component_generics,
			&parse2(quote_spanned!(render_paren.span=> <'a, 'bump: '_>)).unwrap(),
			&render_generics,
			custom_render_args.as_slice(),
			&render_lifetime,
		);

		// FIXME:
		// let constructor_allow_non_camel_case_types = new_has_impl_generics.then(|| {
		// 	quote!(#[allow(non_camel_case_types)])
		// });

		// let render_allow_non_camel_case_types = render_has_impl_generics.then(|| {
		// 	quote!(#[allow(non_camel_case_types)])
		// });

		let constructor_args_field_patterns = constructor_args
			.into_iter()
			.map(|arg| match *arg.argument.fn_arg.pat {
				Pat::Ident(pat_ident) => pat_ident.try_into_field_pat(),
				_ => {
					unreachable!()
				}
			})
			.collect::<Result<Vec<_>>>()?;

		let render_args_field_patterns = render_args
			.into_iter()
			.map(|arg| match *arg.fn_arg.pat {
				Pat::Ident(pat_ident) => pat_ident.try_into_field_pat(),
				_ => {
					unreachable!()
				}
			})
			.collect::<Result<Vec<_>>>()?;

		// These can't be fully hygienic with current technology.
		let new_args_name = Ident::new(
			&format!("{}NewArgs", component_name.to_string()),
			component_name.span(),
		);
		let render_args_name = Ident::new(
			&format!("{}RenderArgs", component_name.to_string()),
			component_name.span(),
		);

		let new_args_builder_name = Ident::new(
			&format!("{}Builder", new_args_name.to_string()),
			component_name.span(),
		);
		let render_args_builder_name = Ident::new(
			&format!("{}Builder", render_args_name.to_string()),
			component_name.span(),
		);

		let render_self: Token![self] = parse2(quote_spanned!(render_paren.span=> self)).unwrap();

		let render_type: ReturnType = match &render_type {
			ReturnType::Default => parse2(quote_spanned! {render_type.span()=>
				-> ::std::result::Result<::#asteracea::lignin::Node<'bump>, ::#asteracea::error::Escalation>
			})
			.unwrap(),
			ReturnType::Type(arrow, type_) => ReturnType::Type(
				*arrow,
				Box::new(Type::Verbatim(
					quote_spanned!(arrow.span()=> ::std::result::Result<#type_, ::#asteracea::error::Escalation>),
				)),
			),
		};

		let constructor_block_statements =
			constructor_block.map(|(_new, _with, block)| block.stmts);

		let call_site_node = Ident::new("node", Span::call_site());

		let (component_impl_generics, component_type_generics, component_where_clause) =
			component_generics.split_for_impl();

		// This (hopefully) enables unused function warnings.
		let new = Ident::new("new", component_name.span());
		let render = Ident::new("render", component_name.span());

		Ok(quote_spanned! {Span::mixed_site()=>
			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::__Asteracea__implementation_details::typed_builder::TypedBuilder)]
			#[builder(doc)]
			// FIXME: #constructor_allow_non_camel_case_types
			#visibility struct #new_args_name#new_args_generics #new_args_body

			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::__Asteracea__implementation_details::typed_builder::TypedBuilder)]
			#[builder(doc)]
			// FIXME: #render_allow_non_camel_case_types
			#visibility struct #render_args_name#render_args_generics #render_args_body

			#(#struct_definition)*

			impl#component_impl_generics #component_name#component_type_generics #component_where_clause {
				#[::#asteracea::trace_escalations(#component_name)]
				#(#constructor_attributes)*
				// FIXME: #constructor_allow_non_camel_case_types
				pub fn #new#new_generics(
					parent_node: &::std::sync::Arc<#asteracea::rhizome::Node>,
					#new_args_name {
						#(#constructor_args_field_patterns,)*
						__Asteracea__phantom: _,
					}: #new_args_name#new_args_generic_args,
				) -> ::std::result::Result<Self, ::#asteracea::error::Escalation> where Self: 'a + 'static { // TODO: Self: 'static is necessary because of `derive_for::<Self>`, but that's not really a good approach... Using derived IDs would be better.
					let #call_site_node = #asteracea::rhizome::extensions::TypeTaggedNodeArc::derive_for::<Self>(parent_node);
					#(#rhizome_extractions)*
					let mut #call_site_node = #call_site_node;

					{} // Isolate constrcutor block.
					#constructor_block_statements
					{} // Dito.

					//FIXME: The eager heap allocation here isn't great.
					// It would probably be sensible to make this lazy through
					// a "seed" or handle placed on the stack that can be referenced
					// by child nodes and derived from further. A dependency extraction
					// or seed lifetime extension would then initialise the backing
					// data structure as needed.
					// I really should add benchmarks before trying this, though.
					let #call_site_node = #call_site_node.into_arc();

					::std::result::Result::Ok(#constructed_value)
				}

				pub fn new_args_builder#new_args_builder_generics()
				-> #new_args_builder_name#new_args_builder_generic_args {
					#new_args_name::builder()
				}

				#[::#asteracea::trace_escalations(#component_name)]
				#(#render_attributes)*
				// FIXME: #render_allow_non_camel_case_types
				pub fn #render#render_generics(
					#render_self: ::std::pin::Pin<&'a Self>,
					#bump: &'bump #asteracea::lignin::bumpalo::Bump,
					#render_args_name {
						#(#render_args_field_patterns,)*
						__Asteracea__phantom: _,
					}: #render_args_name#render_args_generic_args,
				) #render_type {
					let this = #render_self;
					::std::result::Result::Ok(#body)
				}

				pub fn render_args_builder#render_args_builder_generics()
				-> #render_args_builder_name#render_args_builder_generic_args {
					#render_args_name::builder()
				}

				//TODO: Is it possible to call render_args_builder on the reference instead, somehow?
				#[doc(hidden)] // This
				pub fn __Asteracea__ref_render_args_builder#render_args_builder_generics(&self)
				-> #render_args_builder_name#render_args_builder_generic_args {
					let _ = self;
					#render_args_name::builder()
				}
			}

			#(#random_items)*
		})
	}
}
