use super::{GenerateContext, Part, PartKind};
use crate::{
	asteracea_ident,
	storage_context::{ParseContext, ParseWithContext},
	Configuration,
};
use either::Either;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream, Result},
	spanned::Spanned,
	token::{Brace, Question},
	Error, Ident, LitStr, Token,
};
use syn_mid::Block;
use unquote::unquote;

mod kw {
	use syn::custom_keyword;
	custom_keyword!(with);
	custom_keyword!(scope);
	custom_keyword!(attribute);
}

enum AttributeDefinition {
	Assignment(
		Token![.],
		AttributeKey,
		Option<Token![?]>,
		Token![=],
		AttributeValue,
	),
	RustBlock(Token![.], Block),
}

enum AttributeKey {
	Known(Ident),
	Literal(LitStr),
}

impl ToTokens for AttributeKey {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			AttributeKey::Known(name) => {
				let asteracea = asteracea_ident(name.span());
				(quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
					<dyn ::#asteracea::__::lignin_schema::html::attributes::#name>::NAME
				})
				.to_tokens(tokens)
			}
			AttributeKey::Literal(name) => name.to_tokens(tokens),
		}
	}
}

impl Parse for AttributeKey {
	fn parse(input: ParseStream) -> Result<Self> {
		if let Some(name) = input.parse().unwrap() {
			Ok(AttributeKey::Known(name))
		} else if let Some(name) = input.parse().unwrap() {
			Ok(AttributeKey::Literal(name))
		} else {
			Err(Error::new(
				input.span(),
				"Expected HTML attribute key (str literal)",
			))
		}
	}
}

enum AttributeValue {
	Literal(LitStr),
	Blocked(Block),
}

impl ToTokens for AttributeValue {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			AttributeValue::Literal(l) => l.to_tokens(tokens),
			AttributeValue::Blocked(b) => b.to_tokens(tokens),
		}
	}
}

impl Parse for AttributeValue {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if input.peek(LitStr) {
			AttributeValue::Literal(input.parse().unwrap())
		} else if input.peek(Brace) {
			AttributeValue::Blocked(input.parse().unwrap())
		} else {
			return Err(Error::new(
				input.span(),
				"Expected HTML attribute value (string literal or Rust block)",
			));
		})
	}
}

//TODO: Add a Dynamic(Block) variant.
enum ElementName {
	Custom(LitStr),
	Known(Ident, Option<Ident>),
}

pub(crate) struct HtmlDefinition<C: Configuration> {
	lt: Token![<],
	name: ElementName,
	attributes: Vec<AttributeDefinition>,
	pub parts: Vec<Part<C>>,
}

impl<C: Configuration> ParseWithContext for HtmlDefinition<C> {
	type Output = Self;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self> {
		let lt = input.parse::<Token![<]>()?;
		let name = if let Some(name @ LitStr { .. }) = input.parse().unwrap() {
			if name.value().contains(' ') {
				return Err(Error::new_spanned(
					name,
					"Element names must not contain spaces",
				));
			}
			Either::Left(name) // custom
		} else if let Some(name) = input.parse().unwrap() {
			Either::Right(name) // known
		} else {
			return Err(Error::new(
				input.cursor().span(),
				"Expected identifier or string literal (element name)",
			));
		};

		let attributes = {
			let mut attributes = Vec::new();
			while let Ok(dot) = input.parse::<Token![.]>() {
				attributes.push(if input.peek(Ident) || input.peek(LitStr) {
					let key;
					let question: Option<Token![?]>;
					let eq;
					let value;
					unquote!(input, #key #question #eq #value);
					if question.is_some() && matches!(value, AttributeValue::Literal(_)) {
						return Err(Error::new(
							value.span(),
							format!(
							"Expected Rust block value for optional HTML attribute, but found `{}`",
							value.to_token_stream().to_string(),
						),
						));
					}
					AttributeDefinition::Assignment(dot, key, question, eq, value)
				} else if input.peek(Brace) {
					let mut block: Block = input.parse()?;
					block.brace_token.span = block.brace_token.span.resolved_at(Span::mixed_site());
					AttributeDefinition::RustBlock(dot, block)
				} else {
					return Err(Error::new(
						input.span(),
						"Expected Rust block (Attribute) or an identifier or string literal (HTML attribute name)",
					));
				});
			}
			attributes
		};

		let mut parts = Vec::new();
		while !input.peek(Token![>]) && !input.peek(Token![/]) {
			if let Some(part) = Part::parse_with_context(input, cx)? {
				parts.push(part);
			}
		}

		let name = if input.parse::<Token![/]>().is_ok() {
			match name {
				Either::Left(name) => {
					let close_name: LitStr = input.parse()?;
					// Named close.
					if close_name.value() != name.value() {
						return Err(Error::new_spanned(
							close_name,
							format_args!("Expected {:?}", name.value()),
						));
					}
					ElementName::Custom(name)
				}
				Either::Right(name) => {
					let closing_name: Ident = input.parse()?;
					// Named close.
					if closing_name != name {
						return Err(Error::new_spanned(
							closing_name,
							format_args!("Expected `{}`", name),
						));
					}
					ElementName::Known(name, Some(closing_name))
				}
			}
		} else {
			match name {
				Either::Left(name) => ElementName::Custom(name),
				Either::Right(name) => ElementName::Known(name, None),
			}
		};
		input.parse::<Token![>]>()?;

		Ok(Self {
			lt,
			name,
			attributes,
			parts,
		})
	}
}

impl<C: Configuration> HtmlDefinition<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let Self {
			lt,
			name,
			attributes,
			parts,
		} = self;

		let asteracea = asteracea_ident(lt.span());
		let thread_safety = &cx.thread_safety;

		let bump = Ident::new("bump", lt.span().resolved_at(Span::call_site()));

		let has_optional_attributes = attributes.iter().any(|a| match a {
			AttributeDefinition::Assignment(_, _, Some(Question { .. }), _, _) => true,
			AttributeDefinition::Assignment(_, _, None, _, _) => false,
			AttributeDefinition::RustBlock(_, _) => false,
		});
		let validate_attributes = match name {
			ElementName::Custom(_) => vec![],
			ElementName::Known(tag_name, _) => attributes
				.iter()
				.filter_map(|attribute| match attribute {
					AttributeDefinition::Assignment(_, AttributeKey::Known(name), _, _, _) => {
						// Move validation errors onto the attribute name.
						let tag_name = Ident::new(&tag_name.to_string(), name.span());
						Some(
							quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
								// Already flagged where the attribute name is resolved.
								// Ignored here so a deprecated element isn't warned about on the attribute.
								#[allow(deprecated)]
								::#asteracea::__::lignin_schema::html::attributes::#name::<_>::static_validate_on(
									::#asteracea::__::lignin_schema::html::elements::#tag_name
								);
							},
						)
					}
					_ => None,
				})
				.collect::<Vec<_>>(),
		};
		let attributes = attributes
			.iter()
			.map(|a| match a {
				AttributeDefinition::Assignment(dot, key, question, eq, value) => {
					let span = dot.span.resolved_at(Span::mixed_site());
					match (has_optional_attributes, question) {
						(false, Some(_)) => unreachable!(),
						(true, Some(Question { .. })) => {
							let value = match value {
								AttributeValue::Literal(l) => quote_spanned! (l.span()=> l),
								AttributeValue::Blocked(b) => {
									let stmts = &b.stmts;
									quote_spanned! {b.brace_token.span.resolved_at(Span::mixed_site())=>
										#asteracea::ConditionalAttributeValue::into_str_option({ #stmts })
									}
								}
							};
							quote_spanned! {value.span().resolved_at(Span::mixed_site())=> {
								let name = #key; // Always evaluate this.
								if let Some(value) #eq #value {
									attrs.push(#asteracea::lignin::Attribute {
										name,
										value,
									})
								}
							}}
						}
						(true, None) => quote_spanned! {span=>
							attrs.push(#asteracea::lignin::Attribute {
								name: #key,
								value: #value,
							});
						},
						(false, None) => quote_spanned! {span=>
							#asteracea::lignin::Attribute {
								name: #key,
								value: #value,
							}
						},
					}
				}
				AttributeDefinition::RustBlock(dot, block) => {
					let span = dot.span.resolved_at(Span::mixed_site());
					if has_optional_attributes {
						quote_spanned!(span=> attrs.push(#block))
					} else {
						quote_spanned!(span=> #block)
					}
				}
			})
			.collect::<Vec<_>>();

		let attributes = if has_optional_attributes {
			let capacity = attributes.len();
			quote_spanned! {lt.span.resolved_at(Span::mixed_site())=>
				{
					let mut attrs = #asteracea::bumpalo::collections::Vec::with_capacity_in(#capacity, #bump);
					#(#attributes)*
					attrs.into_bump_slice()
				}
			}
		} else {
			quote_spanned!(lt.span=> &*#bump.alloc_with(|| [#(#attributes),*]))
		};

		let (children, parts): (Vec<_>, Vec<_>) = parts
			.iter()
			.partition(|part| (*part).kind() == PartKind::Child);

		let has_content = !children.is_empty();
		let children = if children.len() == 1 {
			children[0].part_tokens(&cx)?
		} else {
			let mut child_stream = TokenStream::new();
			for child in children.into_iter() {
				let child = child.part_tokens(&cx)?;
				child_stream.extend(quote_spanned! {child.span()=>
					#child,
				});
			}
			quote_spanned! {child_stream.span()=>
				::#asteracea::lignin::Node::Multi::<'bump, #thread_safety>(&*#bump.alloc_try_with(
					|| -> ::std::result::Result<_, ::#asteracea::error::Escalation> {
						::std::result::Result::Ok([#child_stream])
					}
				)?)
			}
		};

		let (event_bindings, parts): (Vec<&Part<C>>, Vec<_>) = parts
			.iter()
			.partition(|part| part.kind() == PartKind::EventBinding);
		let mut event_stream = TokenStream::new();
		for event_binding in event_bindings.into_iter() {
			let event_binding = event_binding.part_tokens(&cx)?;
			event_stream.extend(quote_spanned! {event_binding.span()=>
				#event_binding,
			})
		}
		let event_bindings = quote_spanned! {event_stream.span()=>
			&*#bump.alloc_with(|| [#event_stream])
		};

		assert_eq!(parts.len(), 0);
		Ok(match name {
			ElementName::Custom(name) => {
				quote_spanned! {lt.span.resolved_at(Span::mixed_site())=> {
					let children = #children;
					//TODO: Add MathML and SVG support.
					::#asteracea::lignin::Node::HtmlElement::<'bump, #thread_safety> {
						element: #bump.alloc_with(||
								#asteracea::lignin::Element {
									name: #name,
									creation_options: ::#asteracea::lignin::ElementCreationOptions::new(), //TODO: Add `is` support.
									attributes: #attributes,
									content: children,
									event_bindings: #event_bindings,
								}
							),
						//TODO: Add DOM binding support.
						dom_binding: None,
					}
				}}
			}
			ElementName::Known(name, closing_name) => {
				let validate_has_content = if has_content {
					Some(
						quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
							::#asteracea::__::lignin_schema::HasContent::static_validate_on(::#asteracea::__::lignin_schema::html::elements::#name);
						},
					)
				} else {
					None
				};
				let document_closing = closing_name.as_ref().map(|closing_name| {
					quote_spanned! {closing_name.span().resolved_at(Span::mixed_site())=>
						let _ = ::#asteracea::__::lignin_schema::html::elements::#closing_name;
					}
				});
				quote_spanned! {lt.span.resolved_at(Span::mixed_site())=> {
					let children = #children;
					//TODO: Add MathML and SVG support.
					::#asteracea::lignin::Node::HtmlElement::<'bump, #thread_safety> {
						element: #bump.alloc_with(|| {
							#validate_has_content
							#(#validate_attributes)*
							#document_closing
							//TODO: Validate attributes.
							//TODO: Validate events.

							::#asteracea::lignin::Element {
								name: ::#asteracea::__::lignin_schema::html::elements::#name::TAG_NAME,
								creation_options: ::#asteracea::lignin::ElementCreationOptions::new(), //TODO: Add `is` support.
								attributes: #attributes,
								content: children,
								event_bindings: #event_bindings,
							}
						}),
						//TODO: Add DOM binding support.
						dom_binding: None,
					}
				}}
			}
		})
	}
}
