use super::capture_definition::CaptureDefinition;
use crate::{
	asteracea_ident,
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use either::Either;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
	parenthesized,
	parse::{Parse, ParseStream},
	token::Paren,
	Error, ExprPath, Ident, LitStr, Result, Token,
};
use syn_mid::Block;
use tap::Pipe as _;
use unquote::unquote;

pub mod kw {
	use syn::custom_keyword;
	custom_keyword!(on);
	custom_keyword!(capture);
	custom_keyword!(bubble);
	custom_keyword!(active);
}

pub struct EventBindingDefinition {
	on: kw::on,
	mode: EventMode,
	name: EventName,
	eq: Token![=],
	active: Option<kw::active>,
	handler: Either<(Token![fn], Ident, Paren, Token![self], Ident, Block), ExprPath>,
	component_name: Ident,
	registration_field_name: Ident,
}

enum EventMode {
	None,
	Capture(kw::capture),
	Bubble(kw::bubble),
}
impl Parse for EventMode {
	fn parse(input: ParseStream) -> Result<Self> {
		if let Some(capture) = input.parse().unwrap() {
			Self::Capture(capture)
		} else if let Some(bubble) = input.parse().unwrap() {
			Self::Bubble(bubble)
		} else {
			Self::None
		}
		.pipe(Ok)
	}
}

enum EventName {
	Known(Ident),
	Custom(LitStr),
}
impl Parse for EventName {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(Ident) {
			Ok(Self::Known(input.parse().unwrap()))
		} else if lookahead.peek(LitStr) {
			Ok(Self::Custom(input.parse().unwrap()))
		} else {
			Err(lookahead.error())
		}
	}
}

impl EventBindingDefinition {
	pub fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
	) -> Result<EventBindingDefinition> {
		let on: kw::on;
		unquote! {input,
			#on
			#let mode
			#let name
			#let eq
			#let active
		};

		let handler = {
			if let Some(fn_) = input.parse().unwrap() {
				let handler_name = input.parse()?;
				let args_list;
				let paren = parenthesized!(args_list in input);
				unquote! {&args_list,
					#let self_
					,
					#let event
				};
				if !args_list.is_empty() {
					return Err(Error::new(args_list.span(), "Unexpected token"));
				}
				let body = input.parse()?;
				Either::Left((fn_, handler_name, paren, self_, event, body))
			} else {
				Either::Right(input.parse().map_err(|error| {
					Error::new(error.span(), "Expected `fn` or path of event handler")
				})?)
			}
		};

		let component_name = cx
			.component_name
			.ok_or_else(|| {
				Error::new(
					on.span,
					"Event bindings are only available within full components.",
				)
			})?
			.clone();
		let registration_field_name = cx.storage_context.next_field(on.span);

		let asteracea = asteracea_ident(on.span);
		call2_strict(
			quote_spanned! {on.span=>
				#[allow(non_snake_case)] // This currently has no effect, hence `allow_non_snake_case_on_structure_workaround`.
				|#registration_field_name = ::#asteracea::__Asteracea__implementation_details::lazy_init::Lazy::<
					::#asteracea::lignin::CallbackRegistration::<
						#component_name,
						fn(event: ::#asteracea::lignin::web::Event),
					>
				>::new() |;
			},
			|input| {
				enum EventBindingConfiguration {}
				impl Configuration for EventBindingConfiguration {
					const NAME: &'static str = "component! event binding expression";
					const CAN_CAPTURE: bool = true;
				}
				match CaptureDefinition::<EventBindingConfiguration>::parse_with_context(input, cx)
					.expect("Error parsing internal event binding capture")
				{
					None => (),
					Some(_) => unreachable!(),
				}
			},
		)
		.unwrap();

		Ok(EventBindingDefinition {
			on,
			mode,
			name,
			eq,
			active,
			handler,
			component_name,
			registration_field_name,
		})
	}

	pub fn part_tokens(&self) -> TokenStream {
		let EventBindingDefinition {
			on,
			mode,
			name,
			eq,
			active,
			handler,
			component_name,
			registration_field_name,
		} = self;
		let asteracea = asteracea_ident(on.span);

		quote_spanned!(on.span=> todo!("event binding"))
	}
}
