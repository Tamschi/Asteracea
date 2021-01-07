#![forbid(unsafe_code)]
#![allow(clippy::unneeded_field_pattern)]

extern crate proc_macro;

mod component_declaration;
mod map_message;
mod storage_context;
mod part;
mod storage_configuration;
mod syn_ext;
mod try_parse;

use self::{
	component_declaration::ComponentDeclaration,
	map_message::MapMessage,
	part::{GenerateContext, Part},
	try_parse::TryParse,
};
use lazy_static::lazy_static;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::crate_name;
use quote::{quote, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input, Ident, Result,
};

use syn::Error;

#[proc_macro]
pub fn component(input: TokenStream1) -> TokenStream1 {
	let component_declaration = parse_macro_input!(input as ComponentDeclaration);
	let tokens: TokenStream2 = component_declaration
		.into_tokens()
		.unwrap_or_else(|error| error.to_compile_error());
	tokens.into()
}

struct BumpFormat {
	asteracea: Ident,
	bump_span: Span,
	input: TokenStream2,
}

#[proc_macro]
pub fn bump_format(input: TokenStream1) -> TokenStream1 {
	let bump_format = parse_macro_input!(input as BumpFormat);
	quote!(#bump_format).into()
}

impl Parse for BumpFormat {
	fn parse(input: ParseStream) -> Result<Self> {
		//TODO: This is pretty hacky.
		// Change it to a better location once that feature is stable in proc_macro2.
		let bump_span = input.cursor().span();
		let asteracea = asteracea_ident(bump_span);
		Ok(BumpFormat {
			asteracea,
			bump_span,
			input: input.parse()?,
		})
	}
}

impl ToTokens for BumpFormat {
	fn to_tokens(&self, output: &mut TokenStream2) {
		let BumpFormat {
			asteracea,
			bump_span,
			input,
		} = self;
		let bump = Ident::new("bump", bump_span.resolved_at(Span::call_site()));
		output.extend(quote! {
			#asteracea::lignin_schema::lignin::Node::Text(
				#asteracea::lignin_schema::lignin::bumpalo::format!(in #bump, #input)
					.into_bump_str()
			)
		});
	}
}

enum FragmentConfiguration {}
impl Configuration for FragmentConfiguration {
	const NAME: &'static str = "fragment!";
	const CAN_CAPTURE: bool = false;
}

#[proc_macro]
pub fn fragment(input: TokenStream1) -> TokenStream1 {
	parse_macro_input!(input as Part<FragmentConfiguration>)
		.part_tokens(&GenerateContext::default())
		.unwrap_or_else(|error| error.to_compile_error())
		.into()
}

// TODO: Accept reexported asteracea module made available via `use`.
lazy_static! {
	static ref ASTERACEA_NAME: String =
		crate_name("asteracea").unwrap_or_else(|_| "asteracea".to_owned());
}
fn asteracea_ident(span: Span) -> Ident {
	Ident::new(&*ASTERACEA_NAME, span)
}

/// SEE: <https://github.com/rust-lang/rust/issues/34537#issuecomment-554590043>
mod workaround_module {
	pub trait Configuration {
		const NAME: &'static str;
		const CAN_CAPTURE: bool;
	}
}
use workaround_module::Configuration;

fn warn(location: Span, message: &str) -> Result<()> {
	Err(Error::new(location, message.to_string()))
}
