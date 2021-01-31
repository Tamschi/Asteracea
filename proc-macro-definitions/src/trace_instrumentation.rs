use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{parse::Parse, spanned::Spanned, token::Brace, Token};
use syn_mid::{Block, ItemFn};

use crate::asteracea_ident;

pub struct GuiTraced {
	function: ItemFn,
	pub prefix: Option<TokenStream>,
}

impl Parse for GuiTraced {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			function: input.parse()?,
			prefix: None,
		})
	}
}

impl ToTokens for GuiTraced {
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
					let result: ::std::result::Result<_, ::#asteracea::error::GUIError> = (|| { #statements })();
					result.map_err(|gui_error| gui_error.__Asteracea__with_traced_frame(::std::borrow::Cow::Borrowed(::std::stringify!(#prefix#joiner#name))))
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
