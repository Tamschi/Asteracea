use std::collections::HashSet;

use super::{AttachedAccessExpression, CaptureDefinition};
use crate::parse_with_context::{ParseContext, ParseWithContext};
use call2_for_syn::call2_strict;
use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	group::Parens,
	parse::{Parse, ParseStream},
	parse2, parse_quote,
	token::{Brace, Paren},
	visit_mut::{visit_expr_mut, VisitMut},
	Error, Expr, ExprPath, Ident, Result, Token, Visibility,
};
use syn_mid::Block;
use unquote::unquote;

pub enum Component<C> {
	Instantiated {
		capture: CaptureDefinition<C>,
		attached_access: AttachedAccessExpression,
	},
	Instanced {
		open_span: Span,
		reference: Block,
		render_params: Vec<Parameter>,
	},
}
impl<C> ParseWithContext for Component<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let open_span;
		unquote!(input, #^'open_span <* #$'open_span);

		if input.peek(Brace) {
			let mut reference: Block;
			unquote!(input, #reference);

			// Suppress warning.
			reference.brace_token.span = reference.brace_token.span.resolved_at(Span::mixed_site());

			let mut render_params = vec![];
			loop {
				if input.peek(Token![.]) {
					unquote!(input, #let param);
					render_params.push(param)
				} else if input.peek(Token![>]) {
					unquote!(input, >);
					break;
				} else {
					return Err(Error::new(
						input.span(),
						"Expected .render_arg or `>` (end of child component element)",
					));
				}
			}

			Ok(Self::Instanced {
				open_span,
				reference,
				render_params,
			})
		} else {
			// TypePath actually would lead to a better error message here (regarding ::<> use),
			// but that gobbles up eventual nested child components.
			let path: ExprPath;
			unquote!(input, #path);

			let (field_name, visibility) = if input.peek(Token![priv]) {
				let field_name;
				unquote!(input, priv #field_name);
				(field_name, Visibility::Inherited)
			} else {
				match input
					.parse::<Visibility>()
					.expect("Visibility parsing should always succeed.")
				{
					visibility @ Visibility::Public(_)
					| visibility @ Visibility::Crate(_)
					| visibility @ Visibility::Restricted(_) => (input.parse()?, visibility),
					Visibility::Inherited => (
						Ident::new(
							&format!("__asteracea__custom_{}", cx.custom_child_element_count),
							open_span.resolved_at(Span::mixed_site()),
						),
						Visibility::Inherited,
					),
				}
			};

			let mut new_params: Vec<Parameter> = vec![];
			let mut render_params: Vec<Parameter> = vec![];
			loop {
				if input.peek(Token![*]) {
					unquote!(input, #let param);
					new_params.push(param)
				} else if input.peek(Token![.]) {
					unquote!(input, #let param);
					render_params.push(param)
				} else if input.peek(Token![/]) {
					let closing_name: Ident;
					unquote!(input, /#closing_name>);
					if closing_name != path.path.segments.last().ok_or_else(|| Error::new_spanned(path.clone(), "Strange: This path doesn't contain a last segment... Somehow. It's needed for named element closing, so maybe don't do that here."))?.ident {
						return Err(Error::new_spanned(
							closing_name,
							format!("Expected `{}`", path.path.segments.last().unwrap().ident.to_string()),
						));
					}
					break;
				} else if input.peek(Token![>]) {
					unquote!(input, >);
					break;
				} else {
					return Err(Error::new(
						input.span(),
						if let Some(last) = path.path.segments.last() {
							format!("Expected .render_arg or `/{}>` or `>` (end of child component element)", last.ident.to_string())
						} else {
							"Expected .render_arg or `>` (end of child component element)"
								.to_string()
						},
					));
				}
			}

			cx.custom_child_element_count += 1;

			let new_params = parameter_struct_expression(
				open_span,
				parse2(quote_spanned! (open_span=> #path::new_args_builder())).unwrap(),
				new_params.as_slice(),
			);

			Ok(Self::Instantiated {
			capture: call2_strict(
				quote_spanned! {open_span=>
						|#visibility #field_name = #path::new(&node, #new_params)?|
					},
					|input| CaptureDefinition::<C>::parse_with_context(input, cx),
				)
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
				.unwrap(),
			attached_access: {
				let render_params = parameter_struct_expression(
					open_span.resolved_at(Span::mixed_site()),
					parse2(quote_spanned! (open_span.resolved_at(Span::mixed_site())=> #path::render_args_builder())).unwrap(),
					render_params.as_slice(),
				);
				parse2(quote_spanned! (open_span=> .render(bump, #render_params)))
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?}
		})
		}
	}
}

impl<C> Component<C> {
	pub fn part_tokens(&self) -> TokenStream {
		match self {
			Component::Instantiated {
				capture,
				attached_access,
			} => {
				let mut expr = parse_quote!(#capture#attached_access);
				visit_expr_mut(&mut SelfMassager, &mut expr);
				quote!(#expr)
			}
			Component::Instanced {
				open_span,
				reference,
				render_params,
			} => {
				let binding = quote_spanned!(reference.brace_token.span.resolved_at(Span::mixed_site())=> let reference: &_ = #reference;);
				let bump = quote_spanned!(*open_span=> bump);
				let render_params = parameter_struct_expression(
					open_span.resolved_at(Span::mixed_site()),
					parse2(
						quote_spanned!(open_span.resolved_at(Span::mixed_site())=> reference.__asteracea__ref_render_args_builder()),
					).unwrap(),
					render_params.as_slice(),
				);
				let mut expr = parse2(quote_spanned!(open_span.resolved_at(Span::mixed_site())=> {
					#binding
					reference.render(#bump, #render_params)
				}))
				.unwrap();
				visit_expr_mut(&mut SelfMassager, &mut expr);
				quote!(#expr)
			}
		}
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

pub struct Parameter {
	punct: Punct,
	ident: Ident,
	question: Option<Token![?]>,
	eq: Token![=],
	value: Block,
}

impl Parse for Parameter {
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

impl ToTokens for Parameter {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let value_stmts = &self.value.stmts;
		let value = quote_spanned! (self.value.brace_token.span.resolved_at(Span::mixed_site())=> {#value_stmts});
		match self.question {
			Some(_) => value.to_tokens(tokens),
			None => {
				let dot = quote_spanned!(self.punct.span()=> .);
				let ident = &self.ident;
				quote_spanned! (self.eq.span=> #dot#ident(#value)).to_tokens(tokens)
			}
		}
	}
}

fn parameter_struct_expression(
	fallback_span: Span,
	make_builder: Expr,
	parameters: &[Parameter],
) -> TokenStream {
	let optional_names = parameters
		.iter()
		.filter_map(|parameter| parameter.question.map(|_| parameter.ident.to_string()))
		.collect::<HashSet<_>>();

	if optional_names.is_empty() {
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
					let stmts = &value.stmts;
					// Suppress unused-braces warning:
					let value = quote_spanned! {value.brace_token.span.resolved_at(Span::mixed_site())=>
						{#stmts}
					};
					quote_spanned! {punct.span()=>
						.#ident(#value)
					}
				},
			)
			.collect::<Vec<_>>();
		quote_spanned! {fallback_span=>
			#make_builder#(#parameters)*.build()
		}
	} else {
		let param_idents = parameters
			.iter()
			.enumerate()
			.map(|(i, parameter)| Ident::new(format!("p{}", i).as_str(), parameter.ident.span()))
			.collect::<Vec<_>>();

		let conditional_idents = param_idents
			.iter()
			.zip(parameters.iter())
			.filter_map(|(ident, parameter)| parameter.question.map(|_| ident));

		let (early_names, early_idents) = param_idents
			.iter()
			.zip(parameters.iter())
			.filter_map(|(ident, parameter)| {
				if !optional_names.contains(&parameter.ident.to_string()) {
					Some((&parameter.ident, ident))
				} else {
					None
				}
			})
			.unzip::<_, _, Vec<_>, Vec<_>>();

		let (late_names, late_idents) = param_idents
			.iter()
			.zip(parameters.iter())
			.filter_map(|(ident, parameter)| {
				if optional_names.contains(&parameter.ident.to_string()) {
					Some((&parameter.ident, ident))
				} else {
					None
				}
			})
			.unzip::<_, _, Vec<_>, Vec<_>>();

		todo!()
	}
}
