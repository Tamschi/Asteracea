use super::{AttachedAccessExpression, CaptureDefinition};
use crate::parse_with_context::{ParseContext, ParseWithContext};
use call2_for_syn::call2_strict;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
	parse::ParseStream, parse2, parse_quote, visit_mut::visit_expr_mut, visit_mut::VisitMut, Error,
	Ident, Result, Token,
};
use syn_mid::Block;
use unquote::unquote;

pub struct Component<C> {
	capture: CaptureDefinition<C>,
	attached_access: AttachedAccessExpression,
}
impl<C> ParseWithContext for Component<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let open_span;
		let name: Ident;
		unquote!(input, #^'open_span <* #$'open_span #name);

		let mut new_params = vec![];
		let mut render_params = vec![];
		loop {
			if input.peek(Token![*]) {
				let name: Ident;
				let mut block: Block;
				unquote!(input, *#name = #block);

				// Suppress warning.
				block.brace_token.span = block.brace_token.span.resolved_at(Span::mixed_site());

				new_params.push((name, block))
			} else if input.peek(Token![.]) {
				let name: Ident;
				let mut block: Block;
				unquote!(input, .#name = #block);

				// Suppress warning.
				block.brace_token.span = block.brace_token.span.resolved_at(Span::mixed_site());

				render_params.push((name, block))
			} else if input.peek(Token![/]) {
				let closing_name: Ident;
				unquote!(input, /#closing_name>);
				if closing_name != name {
					return Err(Error::new_spanned(
						closing_name,
						format!("Expected `{}`", name.to_string()),
					));
				}
				break;
			} else if input.peek(Token![>]) {
				unquote!(input, >);
				break;
			} else {
				return Err(Error::new(
					input.span(),
					"Expected *new_arg or .render_arg or end of child component element",
				));
			}
		}

		let field_name = Ident::new(
			&format!("__Asteracea_custom_{}", cx.custom_child_element_count),
			open_span.resolved_at(Span::mixed_site()),
		);
		cx.custom_child_element_count += 1;

		let new_params = new_params
			.into_iter()
			.map(|(name, block)| quote_spanned! (open_span=> .#name(#block)))
			.collect::<Vec<_>>();

		let render_params = render_params
			.into_iter()
			.map(|(name, block)| quote_spanned! (open_span=> .#name(#block)))
			.collect::<Vec<_>>();

		Ok(Self {
			capture: call2_strict(
				quote_spanned! {open_span=>
						|#field_name = #name::new(&node, #name::new_args_builder()#(#new_params)*.build())?|
					},
					|input| CaptureDefinition::<C>::parse_with_context(input, cx),
				)
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
				.unwrap(),
			attached_access: parse2(quote_spanned! {open_span=>
					.render(bump, #name::render_args_builder()#(#render_params)*.build())
				})
				.map_err(|_| Error::new(open_span, "Internal Asteracea error: Child component element didn't produce parseable capture"))?
		})
	}
}

impl<C> Component<C> {
	pub fn part_tokens(&self) -> TokenStream {
		let Self {
			capture,
			attached_access,
		} = self;
		let mut expr = parse_quote!(#capture#attached_access);
		visit_expr_mut(&mut SelfMassager, &mut expr);
		quote!(#expr)
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
