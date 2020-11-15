use self::constructor_argument::ConstructorArgument;
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
	part::GenerateContext,
	warn, Configuration, MapMessage, Part, YankAny,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap as _, DebuglessUnwrapNone as _};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	braced, parenthesized,
	parse::{discouraged, Parse, ParseStream, Result},
	parse2, parse_quote,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Paren,
	Attribute, Error, FnArg, Generics, Ident, Lifetime, PatType, ReturnType, Token, Type,
	Visibility, WherePredicate,
};
use unzip_n::unzip_n;

mod constructor_argument;

unzip_n!(5);
unzip_n!(6);
unzip_n!(7);

pub struct ComponentDeclaration {
	cx: ParseContext,
	attributes: Vec<Attribute>,
	visibility: Visibility,
	component_generics: Option<Generics>,
	component_wheres: Vec<WherePredicate>,
	constructor_attributes: Vec<Attribute>,
	constructor_generics: Option<Generics>,
	constructor_args: Vec<ConstructorArgument>,
	render_attributes: Vec<Attribute>,
	render_generics: Option<Generics>,
	render_paren: Paren,
	render_args: Vec<FnArg>,
	render_type: ReturnType,
	static_shared_new_procedure: Vec<TokenStream>,
	static_shared_render_procedure: Vec<TokenStream>,
	new_procedure: Vec<TokenStream>,
	render_procedure: Vec<TokenStream>,
	body: Part<ComponentRenderConfiguration>,
	rhizome_extractions: Vec<TokenStream>,
}

pub struct TypeLevelFieldTarget(pub Lifetime);

impl Parse for TypeLevelFieldTarget {
	fn parse(input: ParseStream) -> Result<Self> {
		let target: Lifetime = input.parse()?;
		match &target.ident {
			x if x == "NEW" => (),
			x if x == "RENDER" => (),
			_ => return Err(Error::new_spanned(target, "Expected 'NEW or 'RENDER.")),
		};
		Ok(Self(target))
	}
}

impl PartialEq for TypeLevelFieldTarget {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.ident == rhs.0.ident //TODO: Check if this goes by name only.
	}
}

impl TypeLevelFieldTarget {
	fn is_new(&self) -> bool {
		self.0.ident == "NEW"
	}
	fn is_render(&self) -> bool {
		self.0.ident == "RENDER"
	}
}

impl ToTokens for TypeLevelFieldTarget {
	fn to_tokens(&self, output: &mut TokenStream) {
		self.0.to_tokens(output);
	}
}

pub struct TypeLevelFieldDefinition {
	pub targets: Vec<TypeLevelFieldTarget>,
	pub field_definition: FieldDefinition,
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

		let component_generics = if input.peek(Token![<]) {
			Some(input.parse()?)
		} else {
			None
		};

		let mut component_wheres = Vec::new();
		if input.peek(Token![where]) {
			input.parse::<Token![where]>()?;
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
			}
		}

		let constructor_attributes = input.call(Attribute::parse_outer)?;

		let constructor_generics = if input.peek(Token![<]) {
			Some(input.parse()?)
		} else {
			None
		};

		if !input.peek(Paren) {
			let message = "Expected parentheses (constructor arguments).".to_string();
			return Err(Error::new(
				input.cursor().span(),
				if !component_wheres.is_empty() {
					message + "\nDid you forget to end the component where clause with a comma?"
				} else {
					message
				},
			));
		}
		let constructor_args;
		parenthesized!(constructor_args in input);
		let constructor_args = Punctuated::<_, Token![,]>::parse_terminated(&constructor_args)?;
		let constructor_args: Vec<ConstructorArgument> = constructor_args.into_iter().collect();

		let render_attributes = input.call(Attribute::parse_outer)?;

		let render_generics = if input.peek(Token![<]) {
			Some(input.parse()?)
		} else {
			None
		};

		let render_args;
		let render_paren = parenthesized!(render_args in input); //TODO: Map message.
		let render_args: Punctuated<FnArg, Token![,]> = Punctuated::parse_terminated(&render_args)?;
		let render_args = render_args.into_iter().collect();

		let render_type = input.parse()?;

		let mut rhizome_extractions = Vec::new();

		let mut cx = ParseContext {
			component_name: Some(component_name),
			..Default::default()
		};

		let mut render_procedure = Vec::default();

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

		let mut static_shared_new_procedure = Vec::default();
		let mut static_shared_render_procedure = Vec::default();
		let mut new_procedure = Vec::default();
		while input.peek(Token![do]) {
			input.parse::<Token![do]>()?;
			input.parse::<Token![for]>()?;
			let lifetime: TypeLevelFieldTarget = input.parse()?;

			// TODO: This can definitely be abstracted into a try_parse function, which would also be helpful elsewhere.
			let static_token = if input.peek(Token![static]) {
				Some(input.parse::<Token![static]>()?)
			} else {
				None
			};

			let procedure = if lifetime.is_new() {
				if static_token.is_some() {
					&mut static_shared_new_procedure
				} else {
					&mut new_procedure
				}
			} else if lifetime.is_render() {
				if static_token.is_some() {
					&mut static_shared_render_procedure
				} else {
					&mut render_procedure
				}
			} else {
				//TODO: When doing this, also evert the branches above so the order matches the execution order.
				todo!("Refactor into TypeLevelFieldTarget so that this todo! isn't necessary anymore.");
			};

			{
				let contents;
				let brace = braced!(contents in input);
				let contents: TokenStream = contents.parse()?;

				procedure.push(quote_spanned! {brace.span=>
					// The extra tokens are here to prevent useful weirdness.
					// #contents can't be scoped since identifiers must be available later.
					{}
					#contents
					// The documentation comment in the next line causes better error locality.
					#[allow(unused_doc_comments)]
					///
					{}
				});
			}
		}

		let body = loop {
			match Part::parse_with_context(input, &mut cx)? {
				None => (),
				Some(body) => break body,
			}
		};

		// These captures are put at the very end of the constructor since they always move their value.
		for constructor_argument in constructor_args.iter() {
			if let ConstructorArgument {
				capture: constructor_argument::Capture::Yes(visibility),
				fn_arg,
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
			cx,
			attributes,
			visibility,
			component_generics,
			component_wheres,
			constructor_attributes,
			constructor_generics,
			constructor_args,
			render_attributes,
			render_generics,
			render_paren,
			render_args,
			render_type,
			static_shared_new_procedure,
			static_shared_render_procedure,
			new_procedure,
			render_procedure,
			body,
			rhizome_extractions,
		})
	}
}

//TODO: Make sure NewStatics and RenderStatics are properly hidden so that they can't collide.
impl ComponentDeclaration {
	#[allow(clippy::cognitive_complexity)]
	pub fn into_tokens(self) -> Result<TokenStream> {
		let Self {
			cx:
				ParseContext {
					component_name,
					static_shared,
					allow_non_snake_case_on_structure_workaround,
					field_definitions,
					event_binding_count: _,
				},
			attributes,
			visibility,
			component_generics,
			component_wheres,
			constructor_attributes,
			constructor_generics,
			mut constructor_args,
			render_attributes,
			render_generics,
			render_paren,
			render_args,
			render_type,
			static_shared_new_procedure,
			static_shared_render_procedure,
			mut new_procedure,
			render_procedure,
			body,
			rhizome_extractions,
		} = self;

		let component_name = component_name.unwrap();

		let asteracea = asteracea_ident(Span::call_site());

		let component_wheres = {
			let mut stream = TokenStream::new();
			for component_where in component_wheres {
				stream.extend(quote! {
					#component_where,
				})
			}
			if !stream.is_empty() {
				stream = quote! {
					where
						#stream
				};
			}
			stream
		};

		let (new_statics, post_new_statics) = static_shared
			.into_iter()
			.partition::<Vec<_>, _>(|ss| ss.targets.iter().any(TypeLevelFieldTarget::is_new));

		let (render_statics, other_statics) = post_new_statics
			.into_iter()
			.partition::<Vec<_>, _>(|ss| ss.targets.iter().any(TypeLevelFieldTarget::is_render));

		assert!(other_statics.is_empty());

		//TODO: Refactor.
		let (
			new_statics,
			borrow_new_statics_for_render_statics_or_in_new,
			borrow_new_statics_in_render,
		) = if new_statics.is_empty() {
			(None, None, None)
		} else {
			let mut any_render = false;
			let (
				attributes,
				_visibilities, //TODO: Expose these as getter on the component struct if public, or something.
				names,
				types,
				values,
				new_pattern_parts,
				render_pattern_parts,
			) = new_statics
				.into_iter()
				.map(|mut ss| {
					let field_name = ss.field_definition.name;
					(
						ss.field_definition.attributes,
						ss.field_definition.visibility,
						field_name.clone(),
						ss.field_definition.field_type,
						ss.field_definition.initial_value,
						if let Some(target) = ss.targets.yank_any(TypeLevelFieldTarget::is_new) {
							let mut name = field_name.clone();
							name.set_span(target.0.ident.span());
							quote!(#name)
						} else {
							unreachable!();
						},
						if let Some(target) = ss.targets.iter().find(|x| x.is_render()) {
							any_render = true;
							let mut field_name = field_name;
							field_name.set_span(target.0.ident.span());
							Some(quote!(#field_name,))
						} else {
							None
						},
					)
				})
				.unzip_n_vec();
			(
				Some(quote! {
					struct NewStatics {
						#(#(#attributes)* #names: #types,)*
					}

					#asteracea::lazy_static::lazy_static! {
						static ref NEW_STATICS: NewStatics = {
							#(#static_shared_new_procedure)*
							NewStatics {
								#(#names: {#values},)*
							}
						};
					}
				}),
				Some(quote! {
					let NewStatics {
						#(#new_pattern_parts,)*
					} = &*NEW_STATICS;
				}),
				if !any_render {
					None // Don't hit the Once if all is nothing.
				} else {
					Some(quote! {
						let NewStatics {
							#(#render_pattern_parts)*
							..
						} = &*NEW_STATICS;
					})
				},
			)
		};

		//TODO: Refactor.
		let (render_statics, borrow_render_statics_in_render) = if render_statics.is_empty() {
			(None, None)
		} else {
			let (
				attributes,
				_visibilities, //TODO: Expose these as getter on the component struct if public, or something.
				names,
				types,
				values,
				render_pattern_parts,
			) = render_statics
				.into_iter()
				.map(|mut ss| {
					let field_name = ss.field_definition.name;
					(
						ss.field_definition.attributes,
						ss.field_definition.visibility,
						field_name.clone(),
						ss.field_definition.field_type,
						ss.field_definition.initial_value,
						if let Some(target) = ss.targets.yank_any(TypeLevelFieldTarget::is_render) {
							let mut field_name = field_name;
							field_name.set_span(target.0.ident.span());
							quote!(#field_name)
						} else {
							unreachable!();
						},
					)
				})
				.unzip_n_vec();
			(
				Some(quote! {
					struct RenderStatics {
						#(#(#attributes)* #names: #types,)*
					}

					#asteracea::lazy_static::lazy_static! {
						static ref RENDER_STATICS: RenderStatics = {

							// The primary use is in the constructor, so while the identifiers are available here for convenience, that shouldn't raise warnings.
							// If value is to be used only here, then it should go into a `do for 'RENDER static` procedure instead.
							#[allow(unused_variables)]
							#borrow_new_statics_for_render_statics_or_in_new

							#(#static_shared_render_procedure)*

							RenderStatics {
								#(#names: {#values},)*
							}
						};
					}
				}),
				Some(quote! {
					let RenderStatics {
						#(#render_pattern_parts,)*
					} = &*RENDER_STATICS;
				}),
			)
		};

		let allow_non_snake_case_on_structure_workaround =
			if allow_non_snake_case_on_structure_workaround {
				quote! (#[allow(non_snake_case)])
			} else {
				quote!()
			};

		let (field_attributes, field_visibilities, field_names, field_types, field_values) =
			field_definitions
				.into_iter()
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

		let render_generics = {
			let (render_generics_lt, render_generics_params, render_generics_gt) =
				match render_generics {
					Some(Generics {
						lt_token,
						params,
						gt_token,
						where_clause,
					}) => {
						let lt_token = lt_token.unwrap();
						let gt_token = gt_token.unwrap();
						assert!(where_clause.is_none());
						(lt_token, Some(params), gt_token)
					}
					None => (
						parse2(quote_spanned!(render_paren.span=> <)).unwrap(),
						None,
						parse2(quote_spanned!(render_paren.span=> >)).unwrap(),
					),
				};
			quote_spanned!(render_generics_lt.span()=> #render_generics_lt 'a: 'bump, 'bump, #render_generics_params #render_generics_gt)
		};

		let bump = quote_spanned! (render_paren.span.resolved_at(Span::call_site())=>
			bump
		);

		let body = body.part_tokens(&GenerateContext::default())?;

		let constructor_args = constructor_args.into_iter().map(|arg| {
			let mut fn_arg = arg.fn_arg;
			fn_arg.attrs.clear();
			fn_arg
		});

		// These can't be fully hygienic with current technology.
		let new_args_name = Ident::new(
			&format!("{}NewArgs", component_name.to_string()),
			component_name.span(),
		);
		let render_args_name = Ident::new(
			&format!("{}RenderArgs", component_name.to_string()),
			component_name.span(),
		);

		let call_site_node = Ident::new("node", Span::call_site());

		Ok(quote_spanned! {Span::mixed_site()=>
			#new_statics
			#render_statics

			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::typed_builder::TypedBuilder)]
			pub struct #new_args_name {
				// TODO
			}

			//TODO: Doc comment referring to associated type.
			#[derive(#asteracea::typed_builder::TypedBuilder)]
			pub struct #render_args_name {
				// TODO
			}

			#allow_non_snake_case_on_structure_workaround
			#(#attributes)*
			#visibility struct #component_name#component_generics
			#component_wheres
			{
				#(
					#(#field_attributes)*
					#field_visibilities #field_names: #field_types,
				)*
			}

			impl#component_generics #component_name#component_generics
			#component_wheres
			{
				#(#constructor_attributes)*
				fn new(
					parent_node: &::std::sync::Arc<#asteracea::rhizome::Node>,
					args: #new_args_name,
				) -> ::std::result::Result<Self, #asteracea::error::ExtractableResolutionError> {
					#borrow_new_statics_for_render_statics_or_in_new

					let #call_site_node = #asteracea::rhizome::extensions::TypeTaggedNodeArc::derive_for::<Self>(parent_node);
					#(#rhizome_extractions)*
					let mut #call_site_node = #call_site_node;

					#(#new_procedure)*

					let #call_site_node = #call_site_node.into_arc();

					Ok(Self {
						#field_initializers
					})
				}

				#(#render_attributes)*
				fn render<'bump>(
					&self,
					#bump: &'bump #asteracea::lignin_schema::lignin::bumpalo::Bump,
					args: #render_args_name,
				) -> #asteracea::lignin_schema::lignin::Node<'bump> {
					todo!()
				}
			}

			//TODO: Remove this impl.
			// impl#component_generics #component_name#component_generics
			// #component_wheres
			// {
			// 	#(#constructor_attributes)*
			// 	pub fn new#constructor_generics(#(#constructor_args),*) -> Self {
			// 		#borrow_new_statics_for_render_statics_or_in_new
			// 		#(#new_procedure)*
			// 		#constructor_result
			// 	}

			// 	#(#render_attributes)*
			// 	pub fn render#render_generics#render_args -> #render_type {
			// 		//TODO: Captures with overlapping visibility should have their names collide.
			// 		#borrow_new_statics_in_render
			// 		#borrow_render_statics_in_render
			// 		#(#render_procedure)*
			// 		(#body)
			// 	}
			// }
		})
	}
}
