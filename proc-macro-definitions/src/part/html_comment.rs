use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
	parse::{ParseStream, Result},
	spanned::Spanned as _,
	LitStr, Token,
};

use super::GenerateContext;

pub struct HtmlComment {
	opening: TokenStream,
	text: LitStr,
}

impl ParseWithContext for HtmlComment {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		_cx: &mut ParseContext,
	) -> syn::Result<Self::Output> {
		// <!--
		let opening = {
			let lt = input.parse::<Token![<]>()?;
			let bang = input.parse::<Token![!]>()?;
			let dash_1 = input.parse::<Token![-]>()?;
			let dash_2 = input.parse::<Token![-]>()?;
			quote!(#lt #bang #dash_1 #dash_2)
		};

		let text = input.parse()?;

		// -->
		input.parse::<Token![-]>()?;
		input.parse::<Token![-]>()?;
		input.parse::<Token![>]>()?;

		Ok(Self { opening, text })
	}
}

impl HtmlComment {
	pub fn part_tokens(&self, _cx: &GenerateContext) -> Result<TokenStream> {
		let Self { opening, text } = self;

		let asteracea = asteracea_ident(opening.span());

		Ok(quote_spanned! {opening.span()=>
			#asteracea::lignin_schema::lignin::Node::Comment(
				#text
			)
		})
	}
}
