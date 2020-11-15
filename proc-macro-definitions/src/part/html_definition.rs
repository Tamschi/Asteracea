use super::{GenerateContext, Part, PartKind};
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
	Configuration,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
	braced,
	parse::{ParseStream, Result},
	spanned::Spanned,
	token::{Brace, Dot},
	Error, Ident, LitStr, Token,
};

mod kw {
	use syn::custom_keyword;
	custom_keyword!(with);
	custom_keyword!(scope);
	custom_keyword!(attribute);
}

enum AttributeDefinition {
	LiteralWithBlock(Dot, LitStr, TokenStream),
	LiteralWithLiteral(Dot, LitStr, LitStr),
	Expression(TokenStream),
}

enum ElementName {
	Custom(LitStr),
	Known(Ident),
}

pub struct HtmlDefinition<C> {
	lt: Token![<],
	name: ElementName,
	attributes: Vec<AttributeDefinition>,
	pub parts: Vec<Part<C>>,
}

impl<C: Configuration> ParseWithContext for HtmlDefinition<C> {
	type Output = Self;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self> {
		let lt = input.parse::<Token![<]>()?;
		let name = if let Ok(name @ LitStr { .. }) = input.parse() {
			if name.value().contains(' ') {
				return Err(Error::new_spanned(
					name,
					"Element names must not contain spaces",
				));
			}
			ElementName::Custom(name)
		} else if let Ok(name) = input.parse() {
			ElementName::Known(name)
		} else {
			return Err(Error::new(
				input.cursor().span(),
				"Expected identifier or string literal (element name)",
			));
		};

		let attributes = {
			let mut attributes = Vec::new();
			while let Ok(dot) = input.parse::<Token![.]>() {
				let attribute_lookahead = input.lookahead1();
				use AttributeDefinition::*;
				attributes.push(if attribute_lookahead.peek(LitStr) {
					let name = input.parse()?;
					input.parse::<Token![=]>()?;
					if let Ok(text) = input.parse() {
						LiteralWithLiteral(dot, name, text)
					} else {
						LiteralWithBlock(dot, name, {
							let content;
							let brace = braced!(content in input);
							let content: TokenStream = content.parse()?;
							quote_spanned! (brace.span=> {#content})
						})
					}
				} else if attribute_lookahead.peek(Brace) {
					let content;
					let brace = braced!(content in input);
					let content: TokenStream = content.parse()?;
					Expression(quote_spanned!(brace.span=> #content))
				} else {
					return Err(attribute_lookahead.error());
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

		if input.parse::<Token![/]>().is_ok() {
			match &name {
				ElementName::Custom(name) => {
					let close_name: LitStr = input.parse()?;
					// Named close.
					if close_name.value() != name.value() {
						return Err(Error::new_spanned(
							close_name,
							format_args!("Expected {:?}", name.value()),
						));
					}
				}
				ElementName::Known(name) => {
					let close_name: Ident = input.parse()?;
					// Named close.
					if close_name != *name {
						return Err(Error::new_spanned(
							close_name,
							format_args!("Expected `{}`", name),
						));
					}
				}
			}
		}
		input.parse::<Token![>]>()?;

		Ok(Self {
			lt,
			name,
			attributes,
			parts,
		})
	}
}

impl<C> HtmlDefinition<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let Self {
			lt,
			name,
			attributes,
			parts,
		} = self;

		let asteracea = asteracea_ident(lt.span());

		let bump = Ident::new("bump", lt.span().resolved_at(Span::call_site()));

		let cx = GenerateContext {
			scope_definitions: cx.scope_definitions.clone(),
		};

		let mut attributes_stream = TokenStream::new();
		for scope_body in &cx.scope_definitions {
			attributes_stream.extend(quote! {
				#scope_body,
			});
		}

		for attribute in attributes.iter() {
			use AttributeDefinition::*;
			match attribute {
				LiteralWithBlock(dot, attr_name, attr_value) => {
					attributes_stream.extend(quote_spanned! {dot.span=>
						#asteracea::lignin_schema::lignin::Attribute {
							name: #attr_name,
							value: #attr_value,
						},
					})
				}
				LiteralWithLiteral(dot, attr_name, attr_text) => {
					attributes_stream.extend(quote_spanned! {dot.span=>
						#asteracea::lignin_schema::lignin::Attribute {
							name: #attr_name,
							value: #attr_text,
						},
					})
				}
				Expression(expression) => attributes_stream.extend(quote! {
					#expression,
				}),
			}
		}

		let (children, parts): (Vec<_>, Vec<_>) = parts
			.iter()
			.partition(|part| (*part).kind() == PartKind::Child);
		let mut child_stream = TokenStream::new();
		for child in children.into_iter() {
			let child = child.part_tokens(&cx)?;
			child_stream.extend(quote_spanned! {child.span()=>
				#child,
			});
		}
		let children = quote_spanned! {child_stream.span()=>
			&*#bump.alloc_with(|| [#child_stream])
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
			ElementName::Custom(name) => quote_spanned! {lt.span=>
				#asteracea::lignin_schema::lignin::Node::Element(
					#bump.alloc_with(||
						#asteracea::lignin_schema::lignin::Element {
							name: #name,
							attributes: &*#bump.alloc_with(|| [#attributes_stream]),
							content: #children,
							event_bindings: #event_bindings,
						}
					)
				)
			},
			ElementName::Known(name) => quote_spanned! {lt.span=>
				#asteracea::lignin_schema::lignin::Node::Element(
					#bump.alloc_with(||
						#asteracea::lignin_schema::#name(
							&*#bump.alloc_with(|| [#attributes_stream]),
							#children,
							#event_bindings,
						)
					)
				)
			},
		})
	}
}
