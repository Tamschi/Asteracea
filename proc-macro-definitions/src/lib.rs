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
	try_parse::TryParse,
};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use std::cell::RefCell;
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input,
	spanned::Spanned,
	Expr, Ident, Result, Token,
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
	asteracea: Expr,
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
		set_asteracea_crate(input.parse()?);
		input.parse::<Token![,]>()?;

		//TODO: This is pretty hacky.
		// Change it to a better location once that feature is stable in proc_macro2.
		let bump_span = input.cursor().span();
		let asteracea = asteracea_crate();
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
		let bump = Ident::new("bump", bump_span.resolved_at(input.span()));
		output.extend(quote! {
			#asteracea::lignin::Node::Text(
				#asteracea::lignin::bumpalo::format!(in #bump, #input)
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
	let body = parse_macro_input!(input as Part<FragmentConfiguration>)
		.part_tokens(&GenerateContext::default())
		.unwrap_or_else(|error| error.to_compile_error());
	let asteracea = asteracea_crate();
	(quote_spanned! {Span::mixed_site()=>
		((|| -> ::std::result::Result<_, #asteracea::error::Escalation> {
			Ok(#body)
		})())
	})
	.into()
}

/// Iff the `"backtrace"` feature is enabled, instruments a function to add a trace frame of the form "attr_param::function_name".
/// This only works on functions that return `Result<_, Escalation>`.
#[proc_macro_attribute]
pub fn trace_escalations(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
	let attr = parse_macro_input!(attr as TraceEscalationsAttr);
	set_asteracea_crate(attr.asteracea_crate);
	if cfg!(feature = "backtrace") {
		let mut gui_traced = parse_macro_input!(item as Tracing);
		gui_traced.prefix = Some(attr.prefix);
		gui_traced.into_token_stream().into()
	} else {
		item
	}
}

struct TraceEscalationsAttr {
	asteracea_crate: Expr,
	prefix: TokenStream2,
}
impl Parse for TraceEscalationsAttr {
	fn parse(input: ParseStream) -> Result<Self> {
		let asteracea_crate = input.parse()?;
		input.parse::<Token![,]>()?;
		Ok(Self {
			asteracea_crate,
			prefix: input.parse()?,
		})
	}
}

// TODO: Accept reexported asteracea module made available via `use`.
thread_local! {
	static ASTERACEA: RefCell<Option<Expr>> = RefCell::default();
}
fn set_asteracea_crate(crate_expr: Expr) {
	ASTERACEA.with(|asteracea| *asteracea.borrow_mut() = Some(crate_expr))
}
fn asteracea_crate() -> Expr {
	ASTERACEA.with(|asteracea| asteracea.borrow().as_ref().unwrap().clone())
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
