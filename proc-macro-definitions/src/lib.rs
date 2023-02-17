#![forbid(unsafe_code)]
#![allow(clippy::needless_late_init, clippy::unneeded_field_pattern)]

extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, quote_spanned};
use std::iter;
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input,
	spanned::Spanned,
	Error, Ident, Result,
};
use tap::Conv;

mod component_declaration;
mod map_message;
mod part;
mod storage_configuration;
mod storage_context;
mod syn_ext;
mod try_parse;

use self::{
	component_declaration::ComponentDeclaration,
	map_message::MapMessage,
	part::{GenerateContext, Part},
};

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
	let mut tokens = TokenStream2::new();
	bump_format.to_tokens_with_context(
		&mut tokens,
		&GenerateContext {
			thread_safety: quote!(_),
			prefer_thread_safe: None,
		},
	);
	tokens.into()
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

impl BumpFormat {
	fn to_tokens_with_context(&self, output: &mut TokenStream2, cx: &GenerateContext) {
		let BumpFormat {
			asteracea,
			bump_span,
			input,
		} = self;
		let thread_safety = &cx.thread_safety;
		let bump = Ident::new("bump", bump_span.resolved_at(Span::call_site()));
		output.extend(quote! {
			#asteracea::lignin::Node::Text::<#thread_safety> {
				text: #asteracea::bumpalo::format!(in #bump, #input)
					.into_bump_str(),
				dom_binding: None, //TODO?: Add DOM binding support.
			}
		});
	}
}

// TODO: Accept reexported asteracea module made available via `use`.
lazy_static! {
	static ref ASTERACEA_NAME: String = crate_name("asteracea")
		.map(|found| match found {
			FoundCrate::Itself => "asteracea".to_string(), // This happens in tests.
			FoundCrate::Name(name) => name,
		})
		.unwrap_or_else(|_| "asteracea".to_owned());
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

trait FailSoftly<T, E>: Sized {
	fn fail_softly(self, errors: &mut impl Extend<E>, fallback: impl FnOnce() -> T) -> T;
	fn fail_softly_into<E2: From<E>>(
		self,
		errors: &mut (impl IntoIterator<Item = E2> + Extend<E2>),
		fallback: impl FnOnce() -> T,
	) -> T;
}
impl<T, E> FailSoftly<T, E> for std::result::Result<T, E> {
	fn fail_softly(self, errors: &mut impl Extend<E>, fallback: impl FnOnce() -> T) -> T {
		self.unwrap_or_else(|error| {
			errors.extend(iter::once(error));
			fallback()
		})
	}

	fn fail_softly_into<E2: From<E>>(
		self,
		errors: &mut (impl IntoIterator<Item = E2> + Extend<E2>),
		fallback: impl FnOnce() -> T,
	) -> T {
		self.map_err(Into::into).fail_softly(errors, fallback)
	}
}

/// An attribute macro that discards its arguments and returns what it is applied to unchanged.
///
/// Used as stub when another attribute is not to be activated.
#[proc_macro_attribute]
pub fn discard_these_attribute_args(args: TokenStream1, item: TokenStream1) -> TokenStream1 {
	drop(args);
	item
}

/// Returns just an `::asteracea::__::tracing::Span`,
/// preserving [`Span`] location but resolving it at [`Span::mixed_site()`](`Span::mixed_site`).
#[proc_macro]
pub fn fake_span(input: TokenStream1) -> TokenStream1 {
	let span = input
		.conv::<TokenStream2>()
		.span()
		.resolved_at(Span::mixed_site());
	let asteracea = asteracea_ident(span);
	quote_spanned!(span=> ::#asteracea::__::tracing::Span).into()
}

/// Discards all tokens and outputs an empty block instead,
/// preserving [`Span`] location but resolving it at [`Span::mixed_site()`](`Span::mixed_site`).
#[proc_macro]
pub fn empty_block(input: TokenStream1) -> TokenStream1 {
	let span = input
		.conv::<TokenStream2>()
		.span()
		.resolved_at(Span::mixed_site());
	quote_spanned!(span=> {}).into()
}
