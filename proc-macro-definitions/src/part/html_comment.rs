use super::GenerateContext;
use crate::storage_context::{ParseContext, ParseWithContext};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, LitStr};
use unquote::unquote;

pub struct HtmlComment {
	open_span: Span,
	text: LitStr,
}

impl ParseWithContext for HtmlComment {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		_cx: &mut ParseContext,
	) -> syn::Result<Self::Output> {
		let open_span;
		let text;
		unquote!(input, #'open_span <!-- #text -->);
		Ok(Self { open_span, text })
	}
}

impl HtmlComment {
	pub fn part_tokens(&self, cx: &GenerateContext) -> TokenStream {
		let &Self {
			open_span,
			ref text,
		} = self;

		let bump = Ident::new("bump", open_span);
		let substrate = cx.substrate;

		quote_spanned! {open_span.resolved_at(Span::mixed_site())=>
			#substrate::comment(#bump, #text)
		}
	}
}
