use crate::{storage_context::ParseContext, workaround_module::Configuration, BumpFormat};
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parenthesized, parse::ParseStream, parse2, token::Paren, Error, LitStr, Result, Token};

pub fn peek_from(input: ParseStream<'_>) -> bool {
	input.peek(Token![!])
}

pub(crate) fn parse_with_context<C: Configuration>(
	input: ParseStream<'_>,
	_cx: &mut ParseContext,
) -> Result<BumpFormat> {
	let bang: Token![!] = input.parse()?;

	let args = if let Some(format_string) = input.parse::<Option<LitStr>>().expect("infallible") {
		let formatted_args = if input.peek(Paren) {
			let args;
			let _: Paren = parenthesized!(args in input);
			args.parse().unwrap()
		} else {
			TokenStream::new()
		};

		quote_spanned!(bang.span=> #format_string, #formatted_args)
	} else if input.peek(Paren) {
		let formatted_args;
		let _: Paren = parenthesized!(formatted_args in input);
		let formatted_args: TokenStream = formatted_args.parse().expect("infallible");

		quote_spanned!(bang.span=> "{}", #formatted_args)
	} else {
		return Err(Error::new(
			input.span(),
			"Expected format string literal (`\"…\"`) or parentheses (`(…)`).",
		));
	};

	parse2::<BumpFormat>(args)
}
