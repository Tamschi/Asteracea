use crate::TryParse;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	parenthesized,
	parse::{Parse, ParseStream, Result},
	token::Paren,
	Ident, MethodTurbofish, Token,
};
pub struct AttachedAccessExpression {
	call: Option<(Paren, TokenStream)>,
	#[allow(clippy::type_complexity)]
	member_chain: Vec<(
		Token![.],
		Ident,
		Option<(Option<MethodTurbofish>, Paren, TokenStream)>,
	)>,
}

impl Parse for AttachedAccessExpression {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self {
			call: if input.peek(Paren) {
				let args;
				#[allow(clippy::eval_order_dependence)] // Not actually an issue.
				Some((parenthesized!(args in input), args.parse()?))
			} else {
				None
			},
			member_chain: {
				let mut member_chain = Vec::new();
				while input.peek(Token![.]) {
					member_chain.push((
						input.parse()?,
						input.parse()?,
						if input.peek(Token![::]) || input.peek(Paren) {
							let args;
							#[allow(clippy::eval_order_dependence)] // Not actually an issue.
							Some((
								input.try_parse()?,
								parenthesized!(args in input),
								args.parse()?,
							))
						} else {
							None
						},
					))
				}
				member_chain
			},
		})
	}
}

impl ToTokens for AttachedAccessExpression {
	fn to_tokens(&self, output: &mut TokenStream) {
		if let Some((paren, args)) = &self.call {
			output.extend(quote_spanned!(paren.span=> (#args)));
		}
		for (dot, ident, call) in &self.member_chain {
			output.extend(quote!(#dot#ident));
			if let Some((turbofish, paren, args)) = call {
				if turbofish.is_some() {
					todo!("AttachedAccessExpression.to_tokens");
				}
				output.extend(quote_spanned!(paren.span=> (#args)));
			}
		}
	}
}
