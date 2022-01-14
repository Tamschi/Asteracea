use crate::{storage_context::ParseContext, workaround_module::Configuration, BumpFormat};
use quote::quote_spanned;
use syn::{parse::ParseStream, parse2, LitStr, Result, Token};
use syn_mid::Block;

pub fn peek_from(input: ParseStream<'_>) -> bool {
	input.peek(Token![!])
}

pub(crate) fn parse_with_context<C: Configuration>(
	input: ParseStream<'_>,
	_cx: &mut ParseContext,
) -> Result<BumpFormat> {
	let bang: Token![!] = input.parse()?;
	let format_string: LitStr = if input.peek(LitStr) {
		input.parse().unwrap()
	} else {
		LitStr::new("{}", bang.span)
	};
	let arg_block: Block = input.parse()?;
	let formatted_args = arg_block.stmts;
	let args = quote_spanned!(arg_block.brace_token.span=> #format_string, #formatted_args);
	parse2::<BumpFormat>(args)
}
