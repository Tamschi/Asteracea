mod bind;
mod box_expression;
mod bump_format_shorthand;
mod capture_definition;
mod component;
mod content;
mod defer;
mod event_binding;
mod for_;
mod html_comment;
mod html_definition;

pub use component::{BlockParentParameters, ParentParameterParser};

//TODO: Rename module and struct to `element_expression` / `ElementExpression`, factor out text expressions and value expressions.
//TODO: Rust expressions shouldn't automatically be blocks except for ones after `with`.

pub use self::capture_definition::CaptureDefinition;
use self::{
	bind::Bind, box_expression::BoxExpression, component::Component, content::Content,
	defer::Defer, for_::For, html_comment::HtmlComment, html_definition::HtmlDefinition,
};
use crate::{
	asteracea_ident,
	storage_context::{ParseContext, ParseWithContext},
	BumpFormat, Configuration,
};
use core::result::Result as coreResult;
use debugless_unwrap::{DebuglessUnwrap as _, DebuglessUnwrapErr as _};
use event_binding::EventBindingDefinition;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream, Result},
	spanned::Spanned as _,
	token::{Brace, Bracket},
	Attribute, Error, Expr, Generics, Ident, LitStr, Pat, Token, Visibility,
};
use syn_mid::Block;
use tap::Pipe as _;
use unquote::unquote;

#[allow(clippy::large_enum_variant, clippy::type_complexity)]
pub(crate) enum Part<C: Configuration> {
	Bind(Bind<C>),
	Box(BoxExpression<C>),
	BumpFormat(BumpFormat),
	Capture(CaptureDefinition<C>),
	Content(Content),
	Comment(HtmlComment),
	Component(Component<C>),
	Defer(Defer<C>),
	EventBinding(EventBindingDefinition),
	For(For<C>),
	RustBlock(Brace, TokenStream),
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
		Box<Block>,
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

#[derive(PartialEq, Eq)]
enum PartKind {
	Child,
	EventBinding,
}

impl<C: Configuration> Part<C> {
	fn kind(&self) -> PartKind {
		match self {
			Part::Bind(_)
			| Part::Box(_)
			| Part::BumpFormat(_)
			| Part::Capture(_)
			| Part::Comment(_)
			| Part::Component(_)
			| Part::Content(_)
			| Part::Defer(_)
			| Part::For(_)
			| Part::RustBlock(_, _)
			| Part::Html(_)
			| Part::If(_, _, _, _, _, _)
			| Part::Match(_, _, _, _, _)
			| Part::Multi(_, _)
			| Part::Text(_)
			| Part::With(_, _, _) => PartKind::Child,
			Part::EventBinding(_) => PartKind::EventBinding,
		}
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

//TODO: Split this off onto a wrapper (FragmentRootPart?) to avoid confusion.
// Or maybe just parse it differently, if that's not too much of an issue.
impl<C: Configuration> Parse for Part<C> {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let span = input.span();
		Self::parse_with_context(
			input,
			&mut ParseContext::new_fragment(&Visibility::Inherited, &Generics::default()),
			&mut BlockParentParameters,
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
		parent_parameter_parser: &mut dyn ParentParameterParser,
	) -> Result<Self> {
		let span = input.span();
		Self::parse_with_context(input, cx, parent_parameter_parser).and_then(|part| {
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

impl<C: Configuration> ParseWithContext for Part<C> {
	type Output = Option<Self>;
	fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
		parent_parameter_parser: &mut dyn ParentParameterParser,
	) -> Result<Self::Output> {
		let lookahead = input.lookahead1();
		if lookahead.peek(bind::kw::bind) {
			Some(Part::Bind(Bind::parse_with_context(
				input,
				cx,
				parent_parameter_parser,
			)?))
		} else if lookahead.peek(Token![box]) {
			Some(Part::Box(BoxExpression::parse_with_context(
				input,
				cx,
				parent_parameter_parser,
			)?))
		} else if lookahead.peek(Token![..]) {
			Some(Part::Content(Content::parse_with_context(
				input,
				cx,
				parent_parameter_parser,
			)?))
		} else if lookahead.peek(defer::kw::defer) {
			Some(Part::Defer(Defer::parse_with_context(
				input,
				cx,
				parent_parameter_parser,
			)?))
		} else if input.peek(Token![for]) {
			Some(Part::For(For::parse_with_context(
				input,
				cx,
				parent_parameter_parser,
			)?))
		} else if lookahead.peek(LitStr) {
			Some(Part::Text(input.parse()?))
		} else if lookahead.peek(Token![<]) {
			match {
				let input = input.fork();
				input.parse::<Token![<]>().unwrap();
				input.parse::<TokenTree>()?
			} {
				TokenTree::Punct(punct) if punct.as_char() == '!' => Some(Part::Comment(
					HtmlComment::parse_with_context(input, cx, parent_parameter_parser)?,
				)),
				TokenTree::Punct(punct) if punct.as_char() == '*' => Some(Part::Component(
					Component::parse_with_context(input, cx, parent_parameter_parser)?,
				)),
				_ => Some(Part::Html(HtmlDefinition::<C>::parse_with_context(
					input,
					cx,
					parent_parameter_parser,
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
				InitMode::Spread(_) => {
					Part::parse_required_with_context(input, cx, &mut BlockParentParameters)?
				}
			};
			let (else_, else_arm) = if let Some(else_) = input.parse().unwrap() {
				(
					else_,
					match &mut init_mode {
						InitMode::Dyn(_) => {
							todo!("`dyn if else`")
						}
						InitMode::Spread(_) => Part::parse_required_with_context(
							input,
							cx,
							&mut BlockParentParameters,
						)?,
					}
					.pipe(Box::new),
				)
			} else {
				(
					Token![else](if_.span),
					call2_for_syn::call2_strict(quote_spanned!(if_.span=> []), |input| {
						Part::parse_required_with_context(input, cx, &mut BlockParentParameters)
					})
					.debugless_unwrap()
					.unwrap()
					.pipe(Box::new),
				)
			};
			Some(Part::If(
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
			unquote!(input, #let match_ #let on);
			let body;
			let bracket = bracketed!(body in input);
			let arms = {
				let input = &body;
				let mut arms = vec![];
				while !input.is_empty() {
					let attrs = input.call(Attribute::parse_outer)?;
					let pats = {
						let mut pats = vec![];
						#[allow(clippy::blocks_in_if_conditions)]
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
						InitMode::Spread(_) => Part::parse_required_with_context(
							input,
							cx,
							&mut BlockParentParameters,
						)?,
					}
					.pipe(Box::new);
					arms.push((attrs, pats, guard, fat_arrow, part))
				}
				arms
			};
			Some(Part::Match(init_mode, match_, Box::new(on), bracket, arms))
		} else if lookahead.peek(Brace) {
			let expression;
			#[allow(clippy::eval_order_dependence)]
			Some(Part::RustBlock(
				braced!(expression in input),
				expression.parse()?,
			))
		} else if lookahead.peek(capture_definition::kw::pin)
			|| lookahead.peek(Token![#])
			|| lookahead.peek(Token![|])
		{
			if C::CAN_CAPTURE {
				CaptureDefinition::parse_with_context(input, cx, parent_parameter_parser)?
					.map(Part::Capture)
			} else {
				return Err(Error::new(
					lookahead.error().span(),
					format!("Captures are unavailable in this context: {}", C::NAME),
				));
			}
		} else if lookahead.peek(Bracket) {
			let content;
			let bracket = bracketed!(content in input);

			parent_parameter_parser.parse_any(input, cx)?;

			let mut inner_parts = Vec::new();
			while !content.is_empty() {
				if let Some(inner_part) =
					Part::<C>::parse_with_context(&content, cx, &mut BlockParentParameters)?
				{
					inner_parts.push(inner_part);
				}
			}
			Some(Part::Multi(bracket, inner_parts))
		} else if lookahead.peek(event_binding::kw::on) {
			Some(Part::EventBinding(
				EventBindingDefinition::parse_with_context(input, cx)?,
			))
		} else if bump_format_shorthand::peek_from(input) {
			Some(Part::<C>::BumpFormat(
				bump_format_shorthand::parse_with_context::<C>(input, cx)?,
			))
		} else if input.peek(kw::with) {
			unquote! {input,
				#let with
				#let block
			};
			let part = Part::parse_with_context(input, cx, parent_parameter_parser)?.map(Box::new);
			Some(Part::With(with, block, part))
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
		}
		.pipe(Ok)
	}
}

pub struct GenerateContext {
	pub thread_safety: TokenStream,
	pub prefer_thread_safe: Option<TokenStream>,
}

impl<C: Configuration> Part<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let thread_safety = &cx.thread_safety;
		let prefer_thread_safe = &cx.prefer_thread_safe;
		let mut part_tokens = match self {
			Part::Bind(bind) => bind.part_tokens(cx)?,
			Part::Box(box_expression) => box_expression.part_tokens(cx)?,
			Part::BumpFormat(bump_format) => {
				let mut tokens = TokenStream::new();
				bump_format.to_tokens_with_context(&mut tokens, cx);
				tokens
			}
			Part::Comment(html_comment) => html_comment.part_tokens(),
			Part::Component(component) => component.part_tokens(cx)?,
			Part::Content(content) => content.part_tokens(),
			Part::Defer(defer) => defer.part_tokens(cx)?,
			Part::For(for_) => for_.part_tokens(cx)?,
			Part::Text(lit_str) => {
				let asteracea = asteracea_ident(lit_str.span());
				quote_spanned! {lit_str.span()=>
					::#asteracea::lignin::Node::Text::<'bump, #thread_safety> {
						text: #lit_str,
						dom_binding: None, //TODO: Add text dom binding support.
					}
				}
			}
			Part::Html(html_definition) => html_definition.part_tokens(cx)?,
			Part::If(InitMode::Dyn(_dyn_), _if_, _condition, _then_part, _else_, _else_part) => {
				todo!("`dyn if`")
			}
			Part::If(InitMode::Spread(_spread), if_, condition, then_part, else_, else_part) => {
				let asteracea = asteracea_ident(if_.span);
				let then_tokens = then_part.part_tokens(cx)?;
				let else_tokens = {
					let else_part = else_part.part_tokens(cx)?;
					quote_spanned!(else_.span().resolved_at(Span::mixed_site())=> ::core::convert::identity( #else_part ))
				};
				quote_spanned!(if_.span.resolved_at(Span::mixed_site())=> {
					let if_: ::#asteracea::lignin::Node::<'bump, #thread_safety> = if #condition {
						::#asteracea::lignin::auto_safety::Align::align(#then_tokens)
					} else {
						::#asteracea::lignin::auto_safety::Align::align(#else_tokens)
					};
					if_
				})
			}
			Part::Match(InitMode::Dyn(_dyn_), _match_, _on, _bracket, _arms) => {
				todo!("`dyn match`")
			}
			Part::Match(InitMode::Spread(_spread), match_, on, bracket, arms) => {
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
				quote_spanned!(match_.span=> #match_ #on #body #prefer_thread_safe)
			}
			Part::RustBlock(brace, statements) => {
				// Why not just parentheses? Because those could be turned into a tuple.
				// Making each of these a full Rust block is a bit strange too, but likely the lesser issue.
				quote_spanned!(brace.span.resolved_at(Span::mixed_site())=> { #statements })
			}
			Part::Capture(capture) => quote!(#capture),
			Part::Multi(bracket, m) => {
				let asteracea = asteracea_ident(bracket.span);
				let m = m
					.iter()
					.map(|part| part.part_tokens(cx))
					.collect::<coreResult<Vec<_>, _>>()?;
				let bump = Ident::new("bump", bracket.span.resolved_at(Span::call_site()));
				quote_spanned! {bracket.span=>
					::#asteracea::lignin::Node::Multi::<'bump, #thread_safety>(&*#bump.alloc_try_with(
						|| -> ::std::result::Result::<_, ::#asteracea::error::Escalation> { ::std::result::Result::Ok([
							#(
								::#asteracea::lignin::auto_safety::Align::align(#m),
							)*
						])}
					)?)
				}
			}
			Part::EventBinding(definition) => definition.part_tokens(),
			Part::With(with, block, part) => {
				let isolate = quote_spanned!(with.span=> {});
				let statements = &block.stmts;
				let part_tokens = part.as_ref().map(|part| part.part_tokens(cx)).transpose()?;
				quote_spanned!(block.brace_token.span=> {
					#statements
					#isolate
					#part_tokens
				})
			}
		};
		cx.prefer_thread_safe.to_tokens(&mut part_tokens);
		Ok(part_tokens)
	}
}
