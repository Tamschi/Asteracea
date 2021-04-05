use super::capture_definition::CaptureDefinition;
use crate::{
	asteracea_ident,
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use either::Either;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parenthesized,
	parse::{Parse, ParseStream},
	spanned::Spanned,
	token::Paren,
	Error, ExprPath, Ident, LitStr, Pat, Result, Token,
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
	handler: Either<(Token![fn], Ident, Paren, Token![self], Pat, Block), ExprPath>,
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
impl ToTokens for EventName {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			EventName::Known(name) => {
				let asteracea = asteracea_ident(name.span());
				(quote_spanned! {name.span().resolved_at(Span::mixed_site())=>
					<dyn ::#asteracea::__Asteracea__implementation_details::lignin_schema::events::#name::<_> as ::#asteracea::__Asteracea__implementation_details::lignin_schema::EventInfo>::NAME
				})
				.to_tokens(tokens)
			}
			EventName::Custom(name) => name.to_tokens(tokens),
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
		let self_ = quote_spanned!(on.span=> self);

		let handler = match handler {
			Either::Left((fn_, handler_name, _, self_, event, body)) => {
				quote_spanned!(fn_.span.resolved_at(Span::mixed_site())=> {
					impl #component_name {
						fn #handler_name(#self_: ::std::pin::Pin<&Self>, #event: ::#asteracea::lignin::web::Event) #body
					}

					unsafe {
						//SAFETY: Defined with a compatible signature directly above.
						// `Pin` is transparent, `&Self` is compatible with `*const Self` for valid pointers.
						::std::mem::transmute(Self::#handler_name as fn(_, _))
					}
				})
			}
			Either::Right(predefined) => {
				quote_spanned!(predefined.span().resolved_at(Span::mixed_site())=> {
					// Deny using component state, since this isn't evaluated more than once.
					let _: fn() = || {
						// Make sure the signature matches
						let _: &dyn ::#asteracea::__Asteracea__implementation_details::CallbackHandler::<Self, ::#asteracea::lignin::web::Event, _> = &#predefined;
					};
					// Make sure it's a function, not a closure
					let handler: fn(_, _) = #predefined;
					unsafe {
						// SAFETY: This is validated to be
						// - signature-matching (via the trait implementation)
						// - not a closure (via the coercion above)
						::std::mem::transmute(handler)
					}
				})
			}
		};

		//TODO: Mode.
		//TODO: Validate mode.
		let active = active.map(|active| quote_spanned!(active.span=> .with_passive(false)));

		quote_spanned!(on.span.resolved_at(Span::mixed_site())=> {
			let registration = this.#registration_field_name.get_or_create(|| {
				::#asteracea::lignin::CallbackRegistration::<Self, fn(::#asteracea::lignin::web::Event)>::new(
					#self_,
					#handler,
				)
			});

			::#asteracea::lignin::EventBinding {
				name: #name,
				options: ::#asteracea::lignin::EventBindingOptions::new()#active, //TODO
				callback: {
					use ::#asteracea::lignin::{
						auto_safety::Align as _,
						callback_registry::ToRefThreadBoundFallback as _,
					};
					registration.to_ref().align()
				}
			}
		})
	}
}
