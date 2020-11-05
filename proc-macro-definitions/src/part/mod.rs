mod attached_access_expression;
mod bump_format_shorthand;
mod capture_definition;
mod event_binding;
mod html_comment;
mod html_definition;

pub use self::{
	attached_access_expression::AttachedAccessExpression, capture_definition::CaptureDefinition,
};
use self::{html_comment::HtmlComment, html_definition::HtmlDefinition};
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
	Configuration,
};
use core::{cell::RefCell, result::Result as coreResult};
use event_binding::EventBindingDefinition;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream, Result},
	token::Bang,
	token::{Add, Brace, Bracket},
	Error, Ident, LitStr, Token,
};

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
			| PartBody::Expression(_, _)
			| PartBody::Html(_)
			| PartBody::Multi(_, _)
			| PartBody::Text(_) => PartKind::Child,
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

#[allow(clippy::large_enum_variant)]
pub enum PartBody<C> {
	Comment(HtmlComment),
	Text(LitStr),
	Html(HtmlDefinition<C>),
	Expression(Brace, TokenStream),
	Capture(CaptureDefinition<C>),
	Multi(Bracket, Vec<Part<C>>),
	EventBinding(EventBindingDefinition),
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
		use PartBody::*;
		Ok(if lookahead.peek(LitStr) {
			Some(Text(input.parse()?))
		} else if lookahead.peek(Token![<]) {
			match {
				let input = input.fork();
				input.parse::<Token![<]>().unwrap();
				input.parse::<Token![!]>().ok()
			} {
				Some(Bang { .. }) => Some(Comment(HtmlComment::parse_with_context(input, cx)?)),
				None => Some(Html(HtmlDefinition::<C>::parse_with_context(input, cx)?)),
			}
		} else if lookahead.peek(Brace) {
			let expression;
			#[allow(clippy::eval_order_dependence)]
			Some(Expression(
				braced!(expression in input),
				expression.parse()?,
			))
		} else if lookahead.peek(Token![#]) || lookahead.peek(Token![|]) {
			if C::CAN_CAPTURE {
				CaptureDefinition::parse_with_context(input, cx)?.map(Capture)
			} else {
				return Err(Error::new(
					lookahead.error().span(),
					format!("Captures are unavailable in this context: {}", C::NAME),
				));
			}
		} else if lookahead.peek(Bracket) {
			cx.imply_bump = true;
			let content;
			let bracket = bracketed!(content in input);
			let mut inner_parts = Vec::new();
			while !content.is_empty() {
				if let Some(inner_part) = Part::<C>::parse_with_context(&content, cx)? {
					inner_parts.push(inner_part);
				}
			}
			Some(Multi(bracket, inner_parts))
		} else if lookahead.peek(Add) {
			Some(EventBinding(EventBindingDefinition::parse_with_context(
				input, cx,
			)?))
		} else if bump_format_shorthand::peek_from(input) {
			bump_format_shorthand::parse_with_context(input, cx)?
		} else {
			return Err(Error::new(
				lookahead.error().span(),
				"Expected one of the following:
                \"text\"
                <element ...>
                {rust expression}
                |declaration: Only = capture|;
                |capture: With = declaration|(and, render, call)
                +\"event_name\" = |event| handler()",
			));
		})
	}
}

#[derive(Default)]
pub struct GenerateContext<'a> {
	scope_definitions: Vec<&'a TokenStream>,
}

impl<C> PartBody<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		use PartBody::*;
		Ok(match self {
			Comment(html_comment) => html_comment.part_tokens(cx)?,
			Text(lit_str) => {
				let asteracea = asteracea_ident(lit_str.span());
				quote_spanned! {lit_str.span()=>
					#asteracea::lignin_schema::lignin::Node::Text(#lit_str)
				}
			}
			Html(html_definition) => html_definition.part_tokens(cx)?,
			Expression(brace, expression) => quote_spanned!(brace.span=> {#expression}),
			Capture(capture) => quote!(#capture),
			Multi(bracket, m) => {
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
			EventBinding(definition) => definition.part_tokens(),
		})
	}
}
