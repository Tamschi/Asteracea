use self::{
	arguments::{Argument, ConstructorArgument},
	parameter_helper_definitions::{CustomArgument, ParameterHelperDefintions},
};
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext, StorageContext},
	part::GenerateContext,
	syn_ext::{AddOptionExt, *},
	warn, Configuration, MapMessage, Part,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap as _, DebuglessUnwrapNone as _};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
	parenthesized,
	parse::{discouraged, Parse, ParseStream, Result},
	parse2,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Paren,
	Attribute, Error, Generics, Ident, Lifetime, Pat, PatIdent, PatType, ReturnType, Token, Type,
	Visibility, WhereClause, WherePredicate,
};
use syn_mid::Block;
use unquote::unquote;
use unzip_n::unzip_n;

mod arguments;
mod parameter_helper_definitions;

unzip_n!(5);
unzip_n!(6);
unzip_n!(7);

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
	render_args: Punctuated<Argument, Token![,]>,
	render_type: ReturnType,
	constructor_block: Option<(kw::new, kw::with, Block)>,
	body: Part<ComponentRenderConfiguration>,
	rhizome_extractions: Vec<TokenStream>,
}

pub struct FieldDefinition {
	pub attributes: Vec<Attribute>,
	pub visibility: Visibility,
	pub name: Ident,
	pub field_type: TokenStream,
	pub initial_value: TokenStream,
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

		let mut cx = ParseContext::new_root(&component_name);

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
				let extracted_semi = quote_spanned!(extracted_question.span=> ;);

				//TODO: Is there a way to write this span more nicely?
				let ref_statement_span = quote!(#ref_token #extracted_for #scope #extracted_name #extracted_colon #extracted_type #extracted_question).span();
				let call_site_node =
					Ident::new("node", ref_statement_span.resolved_at(Span::call_site()));
				rhizome_extractions.push({
					let asteracea = asteracea_ident(ref_statement_span);
					quote_spanned! {
						ref_statement_span=>
						#extracted_let #extracted_name#extracted_colon std::sync::Arc<#extracted_type>
						= <#extracted_type>::extract_from(&#call_site_node)
							.map_err(|error| #asteracea::error::ExtractableResolutionError{
								component: core::any::type_name::<Self>(),
								dependency: core::any::type_name::<#extracted_type>(),
								source: error,
							})#extracted_question#extracted_semi
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
				argument: Argument { fn_arg, .. },
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
			attributes,
			visibility,
			name: component_name,
			storage_context: cx.storage_context,
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
		} = self;

		let asteracea = asteracea_ident(Span::call_site());

		let struct_definition =
			storage_context.type_definition(&visibility, &component_name, &component_generics);

		let (field_attributes, field_visibilities, field_names, field_types, field_values) =
			storage_context
				.field_definitions()
				.map(|c| {
					(
						c.attributes,
						c.visibility,
						c.name,
						c.field_type,
						c.initial_value,
					)
				})
				.unzip_n_vec();

		let field_initializers = quote! {
			#(#field_names: (#field_values),)* // The parentheses around #field_values stop the grammar from breaking as much if no value is provided.
		};

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

		let ParameterHelperDefintions {
			on_parameter_struct: new_args_generics,
			parameter_struct_body: new_args_body,
			on_function: new_generics,
			for_function_args: new_args_generic_args,
			on_builder_function: new_args_builder_generics,
			for_builder_function_return: new_args_builder_generic_args,
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
		} = ParameterHelperDefintions::new(
			&component_generics,
			&parse2(quote_spanned!(render_paren.span=> <'a: 'bump, 'bump: '_>)).unwrap(),
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
				-> #asteracea::lignin_schema::lignin::Node<'bump>
			})
			.unwrap(),
			rt @ ReturnType::Type(_, _) => rt.clone(),
		};

		let constructor_block_statements =
			constructor_block.map(|(_new, _with, block)| block.stmts);

		let call_site_node = Ident::new("node", Span::call_site());

		let (component_impl_generics, component_type_generics, component_where_clause) =
			component_generics.split_for_impl();

		Ok(quote_spanned! {Span::mixed_site()=>
			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::typed_builder::TypedBuilder)]
			#[builder(doc)]
			#visibility struct #new_args_name#new_args_generics #new_args_body

			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::typed_builder::TypedBuilder)]
			#[builder(doc)]
			#visibility struct #render_args_name#render_args_generics #render_args_body

			#(#attributes)*
			#struct_definition

			impl#component_impl_generics #component_name#component_type_generics
			#component_where_clause
			{
				#(#constructor_attributes)*
				pub fn new#new_generics(
					parent_node: &::std::sync::Arc<#asteracea::rhizome::Node>,
					#new_args_name {
						#(#constructor_args_field_patterns,)*
						__asteracea__phantom: _,
					}: #new_args_name#new_args_generic_args,
				) -> ::std::result::Result<Self, #asteracea::error::ExtractableResolutionError> where Self: 'a + 'static { // TODO: Self: 'static is necessary because of `derive_for::<Self>`, but that's not really a good approach... Using derived IDs would be better.
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

					Ok(Self {
						#field_initializers
					})
				}

				pub fn new_args_builder#new_args_builder_generics()
				-> #new_args_builder_name#new_args_builder_generic_args {
					#new_args_name::builder()
				}

				#(#render_attributes)*
				pub fn render#render_generics(
					&'a #render_self,
					#bump: &'bump #asteracea::lignin_schema::lignin::bumpalo::Bump,
					#render_args_name {
						#(#render_args_field_patterns,)*
						__asteracea__phantom: _,
					}: #render_args_name#render_args_generic_args,
				) #render_type {
					{} // Isolation against inner attributes.
					#body
				}

				pub fn render_args_builder#render_args_builder_generics()
				-> #render_args_builder_name#render_args_builder_generic_args {
					#render_args_name::builder()
				}

				//TODO: Is it possible to call render_args_builder on the reference instead, somehow?
				#[doc(hidden)] // This
				pub fn __asteracea__ref_render_args_builder#render_args_builder_generics(&self)
				-> #render_args_builder_name#render_args_builder_generic_args {
					let _ = self;
					#render_args_name::builder()
				}
			}
		})
	}
}
