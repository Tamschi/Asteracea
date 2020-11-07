use super::GenerateContext;
use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
};
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
	parse::{ParseStream, Result},
	LitStr,
};
use unquote::unquote;

pub struct HtmlComment {
	text: LitStr,
}

impl ParseWithContext for HtmlComment {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		_cx: &mut ParseContext,
	) -> syn::Result<Self::Output> {
		let text;
		unquote!(input, <!-- #text -->);
		Ok(Self { text })
	}
}

impl HtmlComment {
	pub fn part_tokens(&self, _cx: &GenerateContext) -> Result<TokenStream> {
		let Self { text } = self;

		let asteracea = asteracea_ident(text.span());

		Ok(quote_spanned! {text.span()=>
			#asteracea::lignin_schema::lignin::Node::Comment(
				#text
			)
		})
	}
}
