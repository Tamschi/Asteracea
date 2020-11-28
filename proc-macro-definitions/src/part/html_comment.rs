use crate::{
	asteracea_ident,
	parse_with_context::{ParseContext, ParseWithContext},
};
use proc_macro2::{Span, TokenStream};
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
		unquote!(input, #^'open_span <!-- #$'open_span #text -->);
		Ok(Self { open_span, text })
	}
}

impl HtmlComment {
	pub fn part_tokens(&self) -> TokenStream {
		let &Self {
			open_span,
			ref text,
		} = self;

		let asteracea = asteracea_ident(open_span);

		quote_spanned! {open_span=>
			#asteracea::lignin_schema::lignin::Node::Comment(
				#text
			)
		}
	}
}
