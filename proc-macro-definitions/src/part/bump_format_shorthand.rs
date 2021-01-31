use crate::{
	asteracea_crate,
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::DebuglessUnwrap as _;
use quote::quote_spanned;
use syn::{parse::ParseStream, LitStr, Result, Token};
use syn_mid::Block;

use super::PartBody;

pub fn peek_from(input: ParseStream<'_>) -> bool {
	input.peek(Token![!])
}

pub fn parse_with_context<C: Configuration>(
	input: ParseStream<'_>,
	cx: &mut ParseContext,
) -> Result<<PartBody<C> as ParseWithContext>::Output> {
	let bang: Token![!] = input.parse()?;
	let format_string: LitStr = if input.peek(LitStr) {
		input.parse().unwrap()
	} else {
		LitStr::new("{}", bang.span)
	};
	let arg_block: Block = input.parse()?;
	let formatted_args = arg_block.stmts;
	let args = quote_spanned!(arg_block.brace_token.span=> {#format_string, #formatted_args});
	let asteracea = asteracea_crate();
	call2_strict(
		quote_spanned!(bang.span=> {#asteracea::bump_format!#args}),
		|input| PartBody::<C>::parse_with_context(input, cx),
	)
	.debugless_unwrap()
}
