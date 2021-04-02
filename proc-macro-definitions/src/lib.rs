#![forbid(unsafe_code)]
#![allow(clippy::unneeded_field_pattern)]

extern crate proc_macro;

mod component_declaration;
mod map_message;
mod part;
mod storage_configuration;
mod storage_context;
mod syn_ext;
mod trace_instrumentation;
mod try_parse;

use self::{
	component_declaration::ComponentDeclaration,
	map_message::MapMessage,
	part::{GenerateContext, Part},
};
use lazy_static::lazy_static;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::crate_name;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input, Ident, Result,
};

use syn::Error;

fn hook_panics() {
	std::panic::set_hook(Box::new(|panic_info| {
		let location = panic_info.location();

		let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
			s
		} else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
			s.as_str()
		} else {
			"(unknown panic type)"
		};
		eprintln!(
			"Asteracea proc macro panic at {} line {}\n\n{}",
			location.map(|l| l.file()).unwrap_or("None"),
			location
				.map(|l| l.line().to_string())
				.unwrap_or_else(|| "None".to_string()),
			payload
		);
	}))
}

#[proc_macro]
pub fn component(input: TokenStream1) -> TokenStream1 {
	hook_panics();

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
			#asteracea::lignin::Node::Text {
				text: #asteracea::bumpalo::format!(in #bump, #input)
					.into_bump_str(),
				dom_binding: None, //TODO?: Add DOM binding support.
			}
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
	let asteracea = asteracea_ident(Span::mixed_site());
	let body = parse_macro_input!(input as Part<FragmentConfiguration>)
		.part_tokens(&GenerateContext {
			thread_safety: quote!(_),
			prefer_thread_safe: None,
		})
		.unwrap_or_else(|error| error.to_compile_error());
	(quote_spanned! {Span::mixed_site()=>
		((|| -> ::std::result::Result<_, ::#asteracea::error::Escalation> {
			Ok(#body)
		})())
	})
	.into()
}

/// Iff the `"backtrace"` feature is enabled, instruments a function to add a trace frame of the form "attr_param::function_name".
/// This only works on functions that return `Result<_, Escalation>`.
#[proc_macro_attribute]
pub fn trace_escalations(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
	if cfg!(feature = "backtrace") {
		let mut gui_traced = parse_macro_input!(item as Tracing);
		gui_traced.prefix = parse_macro_input!(attr as TokenStream2).into();
		gui_traced.into_token_stream().into()
	} else {
		item
	}
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
use trace_instrumentation::Tracing;
use workaround_module::Configuration;

fn warn(location: Span, message: &str) -> Result<()> {
	Err(Error::new(location, message.to_string()))
}
