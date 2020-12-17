mod attached_access_expression;
mod bump_format_shorthand;
mod capture_definition;
mod component;
mod event_binding;
mod html_comment;
mod html_definition;

//TODO: Renamed module and struct to `element_expression` / `ElementExpression`, factor out text expressions and value expressions.

pub use self::{
	attached_access_expression::AttachedAccessExpression, capture_definition::CaptureDefinition,
};
use self::{component::Component, html_comment::HtmlComment, html_definition::HtmlDefinition};
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
	Configuration,
};
use core::{cell::RefCell, result::Result as coreResult};
use event_binding::EventBindingDefinition;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream, Result},
	token::{Add, Brace, Bracket},
	Error, Ident, LitStr, Token,
};
use syn_mid::Block;

pub struct Part<C> {
	body: PartBody<C>,
	attached_access: AttachedAccessExpression, //TODO: Clean this up.
}

#[derive(PartialEq, Eq)]
enum PartKind {
	Child,
	EventBinding,
}

impl<C> Part<C> {
	fn kind(&self) -> PartKind {
		match self.body {
			PartBody::Capture(_)
			| PartBody::Comment(_)
			| PartBody::Component(_)
			| PartBody::Expression(_, _)
			| PartBody::Html(_)
			| PartBody::Multi(_, _)
			| PartBody::Text(_)
			| PartBody::With(_, _, _) => PartKind::Child,
			PartBody::EventBinding(_) => PartKind::EventBinding,
		}
	}
}

impl<C: Configuration> ParseWithContext for Part<C> {
	type Output = Option<Self>;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		Ok(
			if let Some(body) = PartBody::parse_with_context(input, cx)? {
				Some(Self {
					body,
					attached_access: input.parse()?,
				})
			} else {
				None
			},
		)
	}
}
impl<C> Part<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let body = self.body.part_tokens(cx)?;
		let attached_access = &self.attached_access;
		Ok(quote!(#body#attached_access))
	}
}

mod kw {
	syn::custom_keyword!(with);
}

#[allow(clippy::large_enum_variant)]
pub enum PartBody<C> {
	Comment(HtmlComment),
	Component(Component<C>),
	Text(LitStr),
	Html(HtmlDefinition<C>),
	Expression(Brace, TokenStream),
	Capture(CaptureDefinition<C>),
	Multi(Bracket, Vec<Part<C>>),
	EventBinding(EventBindingDefinition),
	With(kw::with, Block, Option<Box<Part<C>>>),
}

//TODO: Split this off onto a wrapper (FragmetRootPart?) to avoid confusion.
// Or maybe just parse it differently, if that's not too much of an issue.
impl<C: Configuration> Parse for Part<C> {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		Self::parse_with_context(input, &mut Default::default()).and_then(|part| {
			if let Some(part) = part {
				Ok(part)
			} else {
				Err(Error::new(
					Span::call_site(),
					"The top-level part must return a value.",
				))
			}
		})
	}
}

thread_local!(static CX: RefCell<Option<ParseContext>> = RefCell::default());
impl<C: Configuration> ParseWithContext for PartBody<C> {
	type Output = Option<Self>;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let lookahead = input.lookahead1();
		Ok(if lookahead.peek(LitStr) {
			Some(PartBody::Text(input.parse()?))
		} else if lookahead.peek(Token![<]) {
			match {
				let input = input.fork();
				input.parse::<Token![<]>().unwrap();
				input.parse::<TokenTree>()?
			} {
				TokenTree::Punct(punct) if punct.as_char() == '!' => Some(PartBody::Comment(
					HtmlComment::parse_with_context(input, cx)?,
				)),
				TokenTree::Punct(punct) if punct.as_char() == '*' => Some(PartBody::Component(
					Component::parse_with_context(input, cx)?,
				)),
				_ => Some(PartBody::Html(HtmlDefinition::<C>::parse_with_context(
					input, cx,
				)?)),
			}
		} else if lookahead.peek(Brace) {
			let expression;
			#[allow(clippy::eval_order_dependence)]
			Some(PartBody::Expression(
				braced!(expression in input),
				expression.parse()?,
			))
		} else if lookahead.peek(Token![#]) || lookahead.peek(Token![|]) {
			if C::CAN_CAPTURE {
				CaptureDefinition::parse_with_context(input, cx)?.map(PartBody::Capture)
			} else {
				return Err(Error::new(
					lookahead.error().span(),
					format!("Captures are unavailable in this context: {}", C::NAME),
				));
			}
		} else if lookahead.peek(Bracket) {
			let content;
			let bracket = bracketed!(content in input);
			let mut inner_parts = Vec::new();
			while !content.is_empty() {
				if let Some(inner_part) = Part::<C>::parse_with_context(&content, cx)? {
					inner_parts.push(inner_part);
				}
			}
			Some(PartBody::Multi(bracket, inner_parts))
		} else if lookahead.peek(Add) {
			Some(PartBody::EventBinding(
				EventBindingDefinition::parse_with_context(input, cx)?,
			))
		} else if bump_format_shorthand::peek_from(input) {
			bump_format_shorthand::parse_with_context(input, cx)?
		} else if input.peek(kw::with) {
			unquote::unquote! {input,
				#let with
				#let block
			};
			let part = Part::parse_with_context(input, cx)?.map(Box::new);
			Some(PartBody::With(with, block, part))
		} else {
			return Err(Error::new(
				lookahead.error().span(),
				"Expected one of the following:
                \"text\"
                <element …>
                {rust expression}
                |declaration: Only = capture|;
                |capture: With = declaration|(and, render, call)
				+\"event_name\" = |event| handler()
				with { …; } <…>",
			));
		})
	}
}

#[derive(Default)]
pub struct GenerateContext {}

impl<C> PartBody<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		Ok(match self {
			PartBody::Comment(html_comment) => html_comment.part_tokens(),
			PartBody::Component(component) => component.part_tokens(),
			PartBody::Text(lit_str) => {
				let asteracea = asteracea_ident(lit_str.span());
				quote_spanned! {lit_str.span()=>
					#asteracea::lignin_schema::lignin::Node::Text(#lit_str)
				}
			}
			PartBody::Html(html_definition) => html_definition.part_tokens(cx)?,
			PartBody::Expression(brace, expression) => quote_spanned!(brace.span=> {#expression}),
			PartBody::Capture(capture) => quote!(#capture),
			PartBody::Multi(bracket, m) => {
				let asteracea = asteracea_ident(bracket.span);
				let m = m
					.iter()
					.map(|part| part.part_tokens(cx))
					.collect::<coreResult<Vec<_>, _>>()?;
				let bump = Ident::new("bump", bracket.span.resolved_at(Span::call_site()));
				quote_spanned! {bracket.span=>
					#asteracea::lignin_schema::lignin::Node::Multi(&*#bump.alloc_with(|| [
						#(#m,)*
					]))
				}
			}
			PartBody::EventBinding(definition) => definition.part_tokens(),
			PartBody::With(with, block, part) => {
				let isolate = quote_spanned!(with.span=> {});
				let statements = &block.stmts;
				let part_tokens = part.as_ref().map(|part| part.part_tokens(cx)).transpose()?;
				quote_spanned!(block.brace_token.span=> {
					#statements
					#isolate
					#part_tokens
				})
			}
		})
	}
}
