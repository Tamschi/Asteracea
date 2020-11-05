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
	closing: TokenStream,
}

impl ParseWithContext for HtmlComment {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		_cx: &mut ParseContext,
	) -> syn::Result<Self::Output> {
		let opening = {
			let lt = input.parse::<Token![<]>()?;
			let bang = input.parse::<Token![!]>()?;
			let dash_1 = input.parse::<Token![-]>()?;
			let dash_2 = input.parse::<Token![-]>()?;
			quote!(#lt #bang #dash_1 #dash_2)
		};

		let text = input.parse()?;

		let closing = {
			let dash_1 = input.parse::<Token![-]>()?;
			let dash_2 = input.parse::<Token![-]>()?;
			let gt = input.parse::<Token![>]>()?;
			quote! (#dash_1 #dash_2 #gt)
		};

		Ok(Self {
			opening,
			text,
			closing,
		})
	}
}

impl HtmlComment {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let Self {
			opening,
			text,
			closing: _,
		} = self;

		let asteracea = asteracea_ident(opening.span());

		Ok(quote_spanned! {opening.span()=>
			#asteracea::lignin_schema::lignin::Node::Comment(
				#text
			)
		})
	}
}
