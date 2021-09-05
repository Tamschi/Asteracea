use crate::asteracea_ident;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{parse::Parse, spanned::Spanned, token::Brace, Token};
use syn_mid::{Block, ItemFn};

pub struct Tracing {
	function: ItemFn,
	pub prefix: Option<TokenStream>,
}

impl Parse for Tracing {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			function: input.parse()?,
			prefix: None,
		})
	}
}

impl ToTokens for Tracing {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		ItemFn {
			block: {
				let asteracea = asteracea_ident(Span::mixed_site());
				let statements = &self.function.block.stmts;
				let prefix = &self.prefix;
				let joiner = prefix.as_ref().map(|p| if p.is_empty() {
					None
				} else {
					Some(Token![::](Span::mixed_site()))
				}).flatten();
				let name = &self.function.sig.ident;
				let inner = quote_spanned! {self.function.sig.output.span().resolved_at(Span::mixed_site())=>
					let result = ::#asteracea::error::Escalation::catch_any(::std::panic::AssertUnwindSafe(
						// UNWIND-SAFETY: Any panic caught here is resumed when the captured `Caught` is escalated.
						|| { #statements }
					));
					let ok = result.map_err(|caught| caught.__Asteracea__with_traced_frame(::std::borrow::Cow::Borrowed(::std::concat!(
						// Not stringifying in bulk avoids spaces between the tokens.
						::std::stringify!(#prefix),
						::std::stringify!(#joiner),
						::std::stringify!(#name),
					))))?;
					Ok(ok)
				};
				Box::new(Block {
					brace_token: Brace(Span::mixed_site()),
					stmts: inner,
				})
			},
			..self.function.clone()
		}
		.to_tokens(tokens)
	}
}
