use crate::storage_context::{ParseContext, ParseWithContext};
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, spanned::Spanned, Ident, Result, Token};
use super::ParentParameterParser;

pub struct Content {
	//TODO: For multi-use, accept `Token![...]`.
	dot2: Token![..],
}

impl ParseWithContext for Content {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, _cx: &mut ParseContext, _: &mut dyn ParentParameterParser) -> Result<Self::Output> {
		Ok(Self {
			dot2: input.parse()?,
		})
	}
}

impl Content {
	pub fn part_tokens(&self) -> TokenStream {
		let bump = Ident::new("bump", self.dot2.span());
		quote_spanned! {self.dot2.span().resolved_at(Span::mixed_site())=>
			__Asteracea__anonymous_content(#bump)?
		}
	}
}
