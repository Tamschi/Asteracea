use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
	braced,
	parse::{Parse, ParseStream},
	token::Brace,
	Pat, Result,
};

#[derive(Clone)]
pub struct Braced {
	pub brace_token: Brace,
	pub contents: TokenStream,
}

impl Parse for Braced {
	fn parse(input: ParseStream) -> Result<Self> {
		let contents;
		Ok(Self {
			brace_token: braced!(contents in input),
			contents: contents.parse()?,
		})
	}
}

impl ToTokens for Braced {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.brace_token
			.surround(tokens, |tokens| self.contents.to_tokens(tokens))
	}
}

pub struct SinglePat {
	pub pat: Pat,
}

impl Parse for SinglePat {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self {
			pat: Pat::parse_single(input)?,
		})
	}
}
