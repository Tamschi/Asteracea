mod attached_access_expression;
mod box_expression;
mod bump_format_shorthand;
mod capture_definition;
mod component;
mod event_binding;
mod html_comment;
mod html_definition;

//TODO: Renamed module and struct to `element_expression` / `ElementExpression`, factor out text expressions and value expressions.
//TODO: Rust expressions shouldn't automatically be blocks except for ones after `with`.

pub use self::{
	attached_access_expression::AttachedAccessExpression, capture_definition::CaptureDefinition,
};
use self::{
	box_expression::BoxExpression, component::Component, html_comment::HtmlComment,
	html_definition::HtmlDefinition,
};
use crate::{
	asteracea_ident,
	storage_context::{ParseContext, ParseWithContext},
	Configuration,
};
use core::result::Result as coreResult;
use debugless_unwrap::{DebuglessUnwrap as _, DebuglessUnwrapErr as _};
use event_binding::EventBindingDefinition;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream, Result},
	spanned::Spanned as _,
	token::{Add, Brace, Bracket},
	Attribute, Error, Expr, Generics, Ident, LitStr, Pat, Token, Visibility,
};
use syn_mid::Block;
use unquote::unquote;
use wyz::Pipe as _;

pub struct Part<C: Configuration> {
	body: PartBody<C>,
	attached_access: AttachedAccessExpression, //TODO: Clean this up.
}

#[derive(PartialEq, Eq)]
enum PartKind {
	Child,
	EventBinding,
}

impl<C: Configuration> Part<C> {
	fn kind(&self) -> PartKind {
		match self.body {
			PartBody::Box(_)
			| PartBody::Capture(_)
			| PartBody::Comment(_)
			| PartBody::Component(_)
			| PartBody::Expression(_, _)
			| PartBody::Html(_)
			| PartBody::If(_, _, _, _, _, _)
			| PartBody::Match(_, _, _, _, _)
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
impl<C: Configuration> Part<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let body = self.body.part_tokens(cx)?;
		let attached_access = &self.attached_access;
		Ok(quote!(#body#attached_access))
	}
}

mod kw {
	syn::custom_keyword!(with);
	syn::custom_keyword!(spread);
}

pub enum InitMode {
	Dyn(Token![dyn]),
	Spread(kw::spread),
}

impl Parse for InitMode {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if let Some(dyn_) = input.parse().unwrap() {
			InitMode::Dyn(dyn_)
		} else if let Some(spread) = input.parse().unwrap() {
			InitMode::Spread(spread)
		} else {
			return Err(Error::new(
				input.span(),
				"Expected one of `dyn` or `spread`",
			));
		})
	}
}

#[allow(clippy::large_enum_variant, clippy::type_complexity)]
pub enum PartBody<C: Configuration> {
	Box(BoxExpression<C>),
	Capture(CaptureDefinition<C>),
	Comment(HtmlComment),
	Component(Component<C>),
	EventBinding(EventBindingDefinition),
	Expression(Brace, TokenStream),
	Html(HtmlDefinition<C>),
	If(
		InitMode,
		Token![if],
		Expr,
		Box<Part<C>>,
		Token![else],
		Box<Part<C>>,
	),
	Match(
		InitMode,
		Token![match],
		Box<Part<C>>,
		Bracket,
		Vec<(
			Vec<Attribute>,
			Vec<(Option<Token![|]>, Pat)>,
			Option<(Token![if], Expr)>,
			Token![=>],
			Box<Part<C>>,
		)>,
	),
	Multi(Bracket, Vec<Part<C>>),
	Text(LitStr),
	With(kw::with, Block, Option<Box<Part<C>>>),
}

//TODO: Split this off onto a wrapper (FragmetRootPart?) to avoid confusion.
// Or maybe just parse it differently, if that's not too much of an issue.
impl<C: Configuration> Parse for Part<C> {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let span = input.span();
		Self::parse_with_context(
			input,
			&mut ParseContext::new_fragment(&Visibility::Inherited, &Generics::default()),
		)
		.and_then(|part| {
			if let Some(part) = part {
				Ok(part)
			} else {
				Err(Error::new(
					span,
					"The top level part must return a value.", //TODO: Better message or better yet program restructuring.
				))
			}
		})
	}
}

impl<C: Configuration> Part<C> {
	pub fn parse_required_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
	) -> Result<Self> {
		let span = input.span();
		Self::parse_with_context(input, cx).and_then(|part| {
			if let Some(part) = part {
				Ok(part)
			} else {
				Err(Error::new(
					span,
					"This part must return a value.", //TODO: Better message or better yet program restructuring.
				))
			}
		})
	}
}

impl<C: Configuration> ParseWithContext for PartBody<C> {
	type Output = Option<Self>;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let lookahead = input.lookahead1();
		Ok(if lookahead.peek(Token![box]) {
			Some(PartBody::Box(BoxExpression::parse_with_context(input, cx)?))
		} else if lookahead.peek(LitStr) {
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
		} else if input.peek(Token![if]) {
			return Err(input.call(InitMode::parse).debugless_unwrap_err());
		} else if (input.peek(Token![dyn]) || input.peek(kw::spread)) && input.peek2(Token![if]) {
			//FIXME: This will be possible much more nicely once unquote is better.
			let mut init_mode = input.parse()?;
			let if_: Token![if] = input.parse()?;
			let condition_body;
			braced!(condition_body in input);
			let condition = condition_body.parse()?;
			if let Some(unexpected) = condition_body.parse().unwrap() {
				let unexpected: TokenTree = unexpected;
				return Err(Error::new(
					unexpected.span(),
					"Unexpected token in `if` condition",
				));
			}
			let then = match &mut init_mode {
				InitMode::Dyn(_) => {
					todo!("`dyn if`")
				}
				InitMode::Spread(_) => Part::parse_required_with_context(input, cx)?,
			};
			let (else_, else_arm) = if let Some(else_) = input.parse().unwrap() {
				(
					else_,
					match &mut init_mode {
						InitMode::Dyn(_) => {
							todo!("`dyn if else`")
						}
						InitMode::Spread(_) => Part::parse_required_with_context(input, cx)?,
					}
					.pipe(Box::new),
				)
			} else {
				(
					Token![else](if_.span),
					call2_for_syn::call2_strict(quote_spanned!(if_.span=> []), |input| {
						Part::parse_required_with_context(input, cx)
					})
					.debugless_unwrap()
					.unwrap()
					.pipe(Box::new),
				)
			};
			Some(PartBody::If(
				init_mode,
				if_,
				condition,
				Box::new(then),
				else_,
				else_arm,
			))
		} else if input.peek(Token![match]) {
			return Err(input.call(InitMode::parse).debugless_unwrap_err());
		} else if (input.peek(Token![dyn]) || input.peek(kw::spread)) && input.peek2(Token![match])
		{
			//FIXME: This will be possible much more nicely once unquote is better.
			let mut init_mode = input.parse()?;
			unquote!(input, #let match_);
			let on = Part::parse_required_with_context(input, cx)?;
			let body;
			let bracket = bracketed!(body in input);
			let arms = {
				let input = &body;
				let mut arms = vec![];
				while !input.is_empty() {
					let attrs = input.call(Attribute::parse_outer)?;
					let pats = {
						let mut pats = vec![];
						while {
							unquote!(input, #let maybe_pipe #let pat);
							pats.push((maybe_pipe, pat));
							input.peek(Token![|])
						} {}
						pats
					};
					let if_: Option<Token![if]> = input.parse()?;
					let guard = if_
						.map(|if_| Result::Ok((if_, input.parse()?)))
						.transpose()?;
					let fat_arrow = input.parse()?;
					let part = match &mut init_mode {
						InitMode::Dyn(_) => {
							todo!("`dyn match arm`")
						}
						InitMode::Spread(_) => Part::parse_required_with_context(input, cx)?,
					}
					.pipe(Box::new);
					arms.push((attrs, pats, guard, fat_arrow, part))
				}
				arms
			};
			Some(PartBody::Match(
				init_mode,
				match_,
				Box::new(on),
				bracket,
				arms,
			))
		} else if lookahead.peek(Brace) {
			let expression;
			#[allow(clippy::eval_order_dependence)]
			Some(PartBody::Expression(
				braced!(expression in input),
				expression.parse()?,
			))
		} else if lookahead.peek(capture_definition::kw::pin)
			|| lookahead.peek(Token![#])
			|| lookahead.peek(Token![|])
		{
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
			unquote! {input,
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
                ⟦pin⟧ |declaration: Only = capture|;
                |capture: With = declaration|(and, render, call)
				+\"event_name\" = |event| handler()
				with { …; } <…>",
			));
		})
	}
}

#[derive(Default)]
pub struct GenerateContext {}

impl<C: Configuration> PartBody<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		Ok(match self {
			PartBody::Box(box_expression) => box_expression.part_tokens(cx)?,
			PartBody::Comment(html_comment) => html_comment.part_tokens(),
			PartBody::Component(component) => component.part_tokens(),
			PartBody::Text(lit_str) => {
				let asteracea = asteracea_ident(lit_str.span());
				quote_spanned! {lit_str.span()=>
					#asteracea::lignin::Node::Text(#lit_str)
				}
			}
			PartBody::Html(html_definition) => html_definition.part_tokens(cx)?,
			PartBody::If(
				InitMode::Dyn(_dyn_),
				_if_,
				_condition,
				_then_part,
				_else_,
				_else_part,
			) => {
				todo!("`dyn if`")
			}
			PartBody::If(
				InitMode::Spread(_spread),
				if_,
				condition,
				then_part,
				else_,
				else_part,
			) => {
				let then_tokens = then_part.part_tokens(cx)?;
				let else_tokens = {
					let else_part = else_part.part_tokens(cx)?;
					quote_spanned!(else_.span()=> { #else_part })
				};
				quote_spanned! {if_.span.resolved_at(Span::mixed_site())=> if #condition {
					#then_tokens
				} else {
					#else_tokens
				}}
			}
			PartBody::Match(InitMode::Dyn(_dyn_), _match_, _on, _bracket, _arms) => {
				todo!("`dyn match`")
			}
			PartBody::Match(InitMode::Spread(_spread), match_, on, bracket, arms) => {
				let on_tokens = on.part_tokens(cx)?;
				let arms = arms
					.iter()
					.map(|(attrs, pats, guard, fat_arrow, part)| {
						let guard = guard
							.as_ref()
							.map(|(if_, guard)| quote_spanned!(if_.span=> #if_ #guard));
						let part = part.part_tokens(cx)?;
						let (pipes, pats) = pats
							.iter()
							.map(|(pipe, pat)| (pipe.as_ref(), pat))
							.unzip::<_, _, Vec<_>, Vec<_>>();
						Ok(quote_spanned! {fat_arrow.span()=>
							#(#attrs)*
							#(#pipes #pats)* #guard #fat_arrow #part,
						})
					})
					.collect::<Result<Vec<_>>>()?;
				let body = quote_spanned!(bracket.span => { #(#arms)* });
				quote_spanned!(match_.span=> #match_ #on_tokens #body)
			}
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
					::#asteracea::lignin::Node::Multi(&*#bump.alloc_try_with(
						|| -> ::std::result::Result::<_, ::#asteracea::error::Escalation> { ::std::result::Result::Ok([
							#(#m,)*
						])}
					)?)
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
