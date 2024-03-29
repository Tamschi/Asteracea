use std::{collections::HashSet, iter};

use super::{GenerateContext, LetSelf};
use crate::{
	asteracea_ident,
	part::Part,
	storage_context::{ParseContext, ParseWithContext},
	util::Braced,
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse2, parse_quote_spanned,
	spanned::Spanned,
	token::{Brace, Eq, Paren, Question},
	visit_mut::{visit_expr_mut, VisitMut},
	Error, Expr, ExprPath, Ident, Label, Pat, PatIdent, PatTupleStruct, Result, Token, Visibility,
};
use tap::Pipe;
use unquote::unquote;

pub enum Component<C: Configuration> {
	Instantiated {
		open_span: Span,
		path: ExprPath,
		capture: LetSelf<C>,
		render_params: Vec<Parameter<Token![.]>>,
		content_children: Vec<ContentChild<C>>,
	},
	Instanced {
		open_span: Span,
		reference: Braced,
		render_params: Vec<Parameter<Token![.]>>,
		content_children: Vec<ContentChild<C>>,
	},
}
impl<C: Configuration> ParseWithContext for Component<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let open_span;
		unquote!(input, #'open_span <*);

		if input.peek(Brace) {
			let mut reference: Braced;
			unquote!(input, #reference);

			// Suppress warning.
			let dummy: Braced = parse_quote_spanned!(reference.brace_token.span.join().resolved_at(Span::mixed_site())=> {});
			reference.brace_token.span = dummy.brace_token.span;

			let mut render_params = vec![];
			while input.peek(Token![.]) && !input.peek(Token![..]) {
				unquote!(input, #let param);
				render_params.push(param)
			}

			let content_children = parse_content_children(input, cx)?;

			if input.peek(Token![>]) {
				unquote!(input, >);
			} else {
				return Err(Error::new(
					input.span(),
					"Expected .render_arg or content child or `>` (end of child component element)",
				));
			}

			Ok(Self::Instanced {
				open_span,
				reference,
				render_params,
				content_children,
			})
		} else {
			// TypePath actually would lead to a better error message here (regarding ::<> use),
			// but that gobbles up eventual nested child components.
			let path: ExprPath = input.parse()?;

			let dot_await: Option<(Token![.], Token![await])> =
				(input.peek(Token![.]) && input.peek2(Token![await])).then(|| {
					(
						input.parse().expect("unreachable"),
						input.parse().expect("unreachable"),
					)
				});
			let dot_await = dot_await.map(|(dot, await_)| {
				quote_spanned! {await_.span=>
					#dot #await_
				}
			});

			let (field_name, visibility) = if input.peek(Token![priv]) {
				let field_name;
				unquote!(input, priv #field_name);
				(field_name, Visibility::Inherited)
			} else {
				match input
					.parse::<Visibility>()
					.expect("Visibility parsing should always succeed.")
				{
					visibility @ Visibility::Public(_) | visibility @ Visibility::Restricted(_) => {
						(input.parse()?, visibility)
					}
					Visibility::Inherited => (
						cx.storage_context.next_field(open_span),
						Visibility::Inherited,
					),
				}
			};

			let mut new_params: Vec<Parameter<Token![*]>> = vec![];
			while input.peek(Token![*]) && input.peek2(Ident) {
				new_params.push(input.parse()?)
			}

			let mut render_params: Vec<Parameter<Token![.]>> = vec![];
			while input.peek(Token![.]) && input.peek2(Ident) {
				render_params.push(input.parse()?)
			}

			let content_children = parse_content_children(input, cx)?;

			if input.peek(Token![/]) {
				let closing_name: Ident;
				unquote!(input, /#closing_name>);
				if closing_name != path.path.segments.last().ok_or_else(|| Error::new_spanned(path.clone(), "Strange: This path doesn't contain a last segment... Somehow. It's needed for named element closing, so maybe don't do that here."))?.ident {
						return Err(Error::new_spanned(
							closing_name,
							format!("Expected `{}`", path.path.segments.last().unwrap().ident),
						));
					}
			} else if input.peek(Token![>]) {
				unquote!(input, >);
			} else {
				return Err(Error::new(
					input.span(),
					if let Some(last) = path.path.segments.last() {
						format!("Expected .render_arg or content child or `/{}>` or `>` (end of child component element)", last.ident)
					} else {
						"Expected .render_arg or content child or `>` (end of child component element)".to_string()
					},
				));
			}

			let new_params = parameter_struct_expression::<C, Token![*]>(
				None,
				open_span,
				parse2(quote_spanned! (open_span=> #path::new_args_builder()))
					.expect("new_params make_builder"),
				new_params.as_slice(),
				&[],
			)?;

			Ok(Self::Instantiated {
				open_span,
				capture: call2_strict(
					quote_spanned! {open_span=>
						let #visibility self.#field_name = pin #path::new(node.as_ref(), #new_params)#dot_await?;
					},
					|input| LetSelf::<C>::parse_with_context(input, cx),
				)
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?,
				path,
				render_params,
				content_children,
			})
		}
	}
}

fn parse_content_children<C: Configuration>(
	input: ParseStream,
	cx: &mut ParseContext,
) -> Result<Vec<ContentChild<C>>> {
	let mut content_children = vec![];
	while !input.peek(Token![/]) && !input.peek(Token![>]) {
		content_children.push(ContentChild {
			slot: input.parse()?,
			parent_parameters: parse_parent_parameters(input)?,
			part: loop {
				if let Some(part) = Part::<C>::parse_with_context(input, cx)? {
					break part;
				}
			},
		})
	}
	Ok(content_children)
}

impl<C: Configuration> Component<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		match self {
			Component::Instantiated {
				open_span,
				capture,
				path,
				render_params,
				content_children,
			} => {
let render_call= {
	let render_params = parameter_struct_expression(
		Some(cx),
		open_span.resolved_at(Span::mixed_site()),
		parse2(quote_spanned! (open_span.resolved_at(Span::mixed_site())=> #path::render_args_builder())).expect("render_params make_builder 1"),
		render_params.as_slice(),
		content_children.as_slice(),
	)?;
	quote_spanned!(*open_span=> .render(bump, #render_params)?)
};

				let asteracea = asteracea_ident(Span::mixed_site());
				let mut expr = parse2(quote!({
					let rendered = #capture #render_call;

					{
						use ::#asteracea::lignin::auto_safety::{AutoSafe as _, Deanonymize as _};
						#[allow(deprecated)]
						rendered.deanonymize()
					}
				}))
				.expect("Component::Instantiated");
				visit_expr_mut(&mut SelfMassager, &mut expr);
				quote!(#expr)
			}
			Component::Instanced {
				open_span,
				reference,
				render_params,
				content_children,
			} => {
				let asteracea = asteracea_ident(*open_span);
				let binding = quote_spanned!(reference.brace_token.span.join().resolved_at(Span::mixed_site())=> let reference: ::std::pin::Pin<&_> = #reference;);
				let bump = quote_spanned!(*open_span=> bump);
				let render_params = parameter_struct_expression(
					Some(cx),
					open_span.resolved_at(Span::mixed_site()),
					parse2(
						quote_spanned!(open_span.resolved_at(Span::mixed_site())=> reference.__Asteracea__ref_render_args_builder()),
					).expect("render_params make_builder 2"),
					render_params.as_slice(),
					content_children.as_slice(),
				)?;
				let mut expr = parse2(quote_spanned!(open_span.resolved_at(Span::mixed_site())=> {
					#binding
					let rendered = reference.render(#bump, #render_params)?;

					{
						use ::#asteracea::lignin::auto_safety::{AutoSafe as _, Deanonymize as _};
						#[allow(deprecated)]
						rendered.deanonymize()
					}
				}))
				.expect("Component::part_tokens Instanced expr");
				visit_expr_mut(&mut SelfMassager, &mut expr);
				quote!(#expr)
			}
		}.pipe(Ok)
	}
}

//TODO: Find out why this is necessary and possibly a better solution.
struct SelfMassager;
impl VisitMut for SelfMassager {
	fn visit_ident_mut(&mut self, i: &mut Ident) {
		if i == "self" {
			i.set_span(i.span().resolved_at(Span::call_site()))
		}
	}
}

//TODO: Unify all parameter types.
#[derive(Clone)]
pub struct Parameter<P> {
	punct: P,
	ident: Ident,
	question: Option<Question>,
	eq: Eq,
	value: Braced,
}

impl<P: Parse> Parse for Parameter<P> {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote! {input,
			#let punct
			#let ident
			#let question
			#let eq
			#let value
		};
		Ok(Parameter {
			punct,
			ident,
			question,
			eq,
			value,
		})
	}
}

impl<P: Spanned> ToTokens for Parameter<P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let value_stmts = &self.value.contents;
		let value = quote_spanned! (self.value.brace_token.span.join().resolved_at(Span::mixed_site())=> {#value_stmts});
		match self.question {
			Some(_) => todo!("Conditional component parameters."),
			None => {
				let dot = quote_spanned!(self.punct.span()=> .);
				let ident = &self.ident;
				quote_spanned! (self.eq.span=> #dot #ident(#value)).to_tokens(tokens)
			}
		}
	}
}

fn parameter_struct_expression<C: Configuration, P: Spanned>(
	cx: Option<&GenerateContext>,
	fallback_span: Span,
	make_builder: Expr,
	parameters: &[Parameter<P>],
	content_children: &[ContentChild<C>],
) -> Result<TokenStream> {
	if parameters
		.iter()
		.all(|parameter| parameter.question.is_none())
	{
		let parameters = parameters
			.iter()
			.map(
				|Parameter {
				     punct,
				     ident,
				     question: _,
				     eq: _,
				     value,
				 }| {
					let stmts = &value.contents;
					// Suppress unneeded-braces warning:
					let value = quote_spanned! {value.brace_token.span.join().resolved_at(Span::mixed_site())=>
						{#stmts}
					};
					quote_spanned! {punct.span()=>
						.#ident(#value)
					}
				},
			)
			.collect::<Vec<_>>();
		let content_children = content_children
			.iter()
			.map(|content_child| content_child.parameter_tokens(cx.expect("`GenerateContent` is required here.")))
			.collect::<Result<Vec<_>>>()?;
		quote_spanned! {fallback_span=>
			#make_builder
				#(#parameters)*
				#(#content_children)*
				.build()
		}
	} else {
		if !content_children.is_empty() {
			todo!("Content children in the presence of optional parameters")
		}

		let mut deferred_names = HashSet::new();

		let mut deferred = vec![];

		let mut output = quote_spanned! {fallback_span=>
			let builder = #make_builder;
		};

		for parameter in parameters.iter() {
			let stmts = &parameter.value.contents;
			// Suppress unneeded-braces warning.
			let value = quote_spanned! {parameter.value.brace_token.span.join().resolved_at(Span::mixed_site())=> {#stmts}};
			if parameter.question.is_some() {
				deferred_names.insert(parameter.ident.to_string());
			}
			output.extend(if deferred_names.contains(&parameter.ident.to_string()) {
				let ident = Ident::new(
					&format!("deferred_parameter_{}", deferred.len()),
					parameter.ident.span().resolved_at(Span::mixed_site()),
				);
				deferred.push(Deferred {
					name: parameter.ident.clone(),
					deferred: ident.clone(),
					conditional: parameter.question.is_some(),
				});
				quote_spanned! {parameter.punct.span().resolved_at(Span::mixed_site())=>
					let #ident = #value;
				}
			} else {
				let name = &parameter.ident;
				quote_spanned! {parameter.punct.span().resolved_at(Span::mixed_site())=>
					let builder = builder.#name(#value);
				}
			})
		}

		let conditional_idents = deferred
			.iter()
			.filter(|deferred| deferred.conditional)
			.map(|deferred| &deferred.deferred)
			.collect::<Vec<_>>();

		let mut match_arms: Box<dyn '_ + CloneableIterator<'_, Item = MatchArm<'_>>> =
			Box::new(iter::once(MatchArm {
				pattern_content: Box::new(iter::empty()),
				builder_calls: Box::new(iter::empty()),
			}));

		for deferred in deferred.iter() {
			if deferred.conditional {
				match_arms = Box::new(match_arms.flat_map(move |previous| {
					iter::once(MatchArm {
						pattern_content: Box::new(
							previous.pattern_content.clone().chain(iter::once((
								Pat::Ident(PatIdent {
									attrs: vec![],
									by_ref: None,
									mutability: None,
									ident: parse2(
										quote_spanned!(deferred.name.span().resolved_at(Span::mixed_site())=> None),
									)
									.unwrap(),
									subpat: None,
								}),
								Token![,](deferred.name.span()),
							))),
						),
						builder_calls: previous.builder_calls.clone(),
					})
					.chain(iter::once(MatchArm {
						pattern_content: Box::new(
							previous.pattern_content.clone().chain(iter::once((
								Pat::TupleStruct(PatTupleStruct {
									attrs: vec![],
									qself: None,
									path: parse2(
										quote_spanned!(deferred.name.span().resolved_at(Span::mixed_site())=> Some),
									)
									.expect("conditional parameter Some"),
									paren_token: Paren(
										deferred.name.span().resolved_at(Span::mixed_site()),
									),
									elems: iter::once(Pat::Ident(PatIdent {
										attrs: vec![],
										by_ref: None,
										mutability: None,
										ident: deferred.deferred.clone(),
										subpat: None,
									}))
									.collect(),
								}),
								Token![,](deferred.name.span()),
							))),
						),
						builder_calls: Box::new(previous.builder_calls.chain(iter::once(
							BuilderMethodCall {
								builder_method: deferred.name.clone(),
								deferred: deferred.deferred.clone(),
							},
						))),
					}))
				}));
			} else {
				match_arms = Box::new(match_arms.map(move |previous| MatchArm {
					pattern_content: previous.pattern_content,
					builder_calls: Box::new(previous.builder_calls.chain(iter::once(
						BuilderMethodCall {
							builder_method: deferred.name.clone(),
							deferred: deferred.deferred.clone(),
						},
					))),
				}))
			}
		}

		let match_arms = match_arms.map(
			|MatchArm {
			     pattern_content,
			     builder_calls,
			 }| {
				let (pats, commata) = pattern_content.unzip::<_, _, Vec<_>, Vec<_>>();
				let (method_names, values) = builder_calls
					.map(|builder_call| (builder_call.builder_method, builder_call.deferred))
					.unzip::<_, _, Vec<_>, Vec<_>>();
				quote_spanned! {fallback_span.resolved_at(Span::mixed_site())=>
					(#(#pats #commata )*) => builder #(.#method_names(#values))*.build(),
				}
			},
		);

		quote_spanned! {fallback_span.resolved_at(Span::mixed_site())=> {
			#output
			match (#(#conditional_idents, )*) {
				#(#match_arms)*
			}
		}}
	}.pipe(Ok)
}

pub struct ContentChild<C: Configuration> {
	slot: Slot,
	parent_parameters: Vec<Parameter<Token![->]>>,
	part: Part<C>,
}

enum Slot {
	Anonymous(Span),
	Named(Label),
}

impl Parse for Slot {
	fn parse(input: ParseStream) -> Result<Self> {
		match input.parse()? {
			Some(label) => Self::Named(label),
			None => Self::Anonymous(input.span()),
		}
		.pipe(Ok)
	}
}

impl<C: Configuration> ContentChild<C> {
	fn parameter_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let (span, slot) = match &self.slot {
			Slot::Anonymous(span) => {
				let span = span.resolved_at(Span::mixed_site());
				(span, Ident::new("__Asteracea__anonymous_content", span))
			}
			Slot::Named(label) => (
				label.colon_token.span.resolved_at(Span::mixed_site()),
				label.name.ident.clone(),
			),
		};

		let asteracea = asteracea_ident(span);

		let parent_parameter_tokens = parameter_struct_expression::<C, Token![->]>(
			Some(cx),
			span,
			parse_quote_spanned!(span=> ::#asteracea::__::infer_builder(phantom)),
			&self.parent_parameters,
			&[],
		)?;
		let part = self.part.part_tokens(cx)?;

		let bump = Ident::new("bump", span.resolved_at(Span::call_site()));
		let bump_time = quote_spanned!(bump.span()=> 'bump);
		let part = match &self.part {
			Part::Async(_) => part,
			_ => quote_spanned! {span=>
				::std::boxed::Box::new(
					|#bump: &#bump_time ::#asteracea::bumpalo::Bump| -> ::std::result::Result<_, ::#asteracea::error::Escalation> {
						::core::result::Result::Ok(#part)
					}
				)
			},
		};

		quote_spanned! {span=>
			.#slot((
				{
					// Many thanks to Yandros for help with the type inference here:
					let phantom = [];
					if false {
						<[_; 0] as ::core::iter::IntoIterator>::into_iter(phantom).next().unwrap()
					} else {
						#parent_parameter_tokens
					}
				},
				#part,
			))
		}
		.to_token_stream()
		.pipe(Ok)
	}
}

trait CloneableIterator<'a>: Iterator {
	fn box_clone(&self) -> Box<dyn 'a + CloneableIterator<'a, Item = Self::Item>>;
}
impl<'a, T: 'a + Iterator> CloneableIterator<'a> for T
where
	T: Clone,
{
	fn box_clone(&self) -> Box<dyn 'a + CloneableIterator<'a, Item = Self::Item>> {
		Box::new(self.clone())
	}
}
impl<'a, Item: 'a> Clone for Box<dyn 'a + CloneableIterator<'a, Item = Item>> {
	fn clone(&self) -> Self {
		(**self).box_clone()
	}
}

#[derive(Clone)]
struct Deferred {
	name: Ident,
	deferred: Ident,
	conditional: bool,
}

#[derive(Clone)]
struct MatchArm<'a> {
	pattern_content: Box<dyn 'a + CloneableIterator<'a, Item = (Pat, Token![,])>>,
	builder_calls: Box<dyn 'a + CloneableIterator<'a, Item = BuilderMethodCall>>,
}

#[derive(Clone)]
struct BuilderMethodCall {
	builder_method: Ident,
	deferred: Ident,
}

fn parse_parent_parameters(input: ParseStream) -> Result<Vec<Parameter<Token![->]>>> {
	let mut parameters = vec![];
	while input.peek(Token![->]) {
		parameters.push(input.parse()?);
	}
	Ok(parameters)
}
