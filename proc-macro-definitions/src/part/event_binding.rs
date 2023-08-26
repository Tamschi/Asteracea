use crate::{
	asteracea_ident,
	storage_context::ParseContext,
	util::{Braced, SinglePat},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parenthesized,
	parse::{Parse, ParseStream},
	spanned::Spanned,
	token::Paren,
	Error, ExprPath, Ident, LitStr, Pat, Result, Token,
};
use tap::Pipe as _;
use unquote::unquote;

pub mod kw {
	use syn::custom_keyword;
	custom_keyword!(on);
	custom_keyword!(capture);
	custom_keyword!(bubble);
	custom_keyword!(active);
	custom_keyword!(once);
}

pub struct EventBindingDefinition {
	on: kw::on,
	mode: EventMode,
	name: EventName,
	active: Option<kw::active>,
	once: Option<kw::once>,
	handler: Handler,
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
					<dyn ::#asteracea::__::lignin_schema::events::#name::<_> as ::#asteracea::__::lignin_schema::EventInfo>::NAME
				})
				.to_tokens(tokens)
			}
			EventName::Custom(name) => name.to_tokens(tokens),
		}
	}
}

impl EventName {
	fn to_partial_identifier_string(&self) -> String {
		match self {
			EventName::Known(name) => name.to_string().trim_start_matches("r#").to_string(),
			EventName::Custom(name) => name
				.value()
				.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
		}
	}
}

enum Handler {
	Inline {
		fn_: Token![fn],
		handler_name: Option<Ident>,
		paren: Paren,
		self_: Token![self],
		comma: Token![,],
		event: Pat,
		body: Braced,
	},
	Predefined(ExprPath),
}

impl EventBindingDefinition {
	pub fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
	) -> Result<EventBindingDefinition> {
		let on: kw::on;
		let name: EventName;
		unquote! {input,
			#on
			#let mode
			#name
			=
			#let active
			#let once
		};

		let handler: Handler = {
			if let Some(fn_) = input.parse().unwrap() {
				let handler_name = input.parse()?;
				let args_list;
				let paren = parenthesized!(args_list in input);
				let event: SinglePat;
				unquote! {&args_list,
					#let self_
					#let comma
					#event
				};
				if !args_list.is_empty() {
					return Err(Error::new(args_list.span(), "Unexpected token"));
				}
				let body = input.parse()?;
				Handler::Inline {
					fn_,
					handler_name,
					paren,
					self_,
					comma,
					event: event.pat,
					body,
				}
			} else {
				Handler::Predefined(input.parse().map_err(|error| {
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

		let registration_field_name = Ident::new(
			&format!(
				"__Asteracea__event_binding_{}_on_{}_{}",
				cx.callback_registrations.borrow().len(),
				name.to_partial_identifier_string(),
				match &handler {
					Handler::Inline { handler_name, .. } => {
						handler_name
							.as_ref()
							.map(|handler_name| {
								"run_".to_string()
									+ handler_name.to_string().trim_start_matches("r#")
							})
							.unwrap_or_else(|| "anonymous_inline_handler".to_string())
					}
					Handler::Predefined(path) => {
						path.to_token_stream()
							.to_string()
							.replace(' ', "")
							.replace("::", "_")
							.replace("r#", "")
							.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
					}
				}
			),
			on.span.resolved_at(Span::mixed_site()),
		);

		let asteracea = asteracea_ident(on.span);
		cx.callback_registrations.borrow_mut().push((
			registration_field_name.clone(),
			syn::parse2(quote_spanned! {on.span.resolved_at(Span::mixed_site())=>
				::#asteracea::lignin::web::Event
			})
			.expect("event binding parameter type"),
		));

		Ok(EventBindingDefinition {
			on,
			mode,
			name,
			active,
			once,
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
			active,
			once,
			handler,
			component_name,
			registration_field_name,
		} = self;
		let asteracea = asteracea_ident(on.span);
		let self_ = quote_spanned!(on.span=> self);

		let handler = match handler {
			Handler::Inline {
				fn_,
				handler_name,
				paren,
				self_,
				comma,
				event,
				body,
			} => {
				let handler_name = handler_name.as_ref().cloned().unwrap_or_else(|| {
					let mut handler_name = registration_field_name.clone();
					handler_name.set_span(fn_.span.resolved_at(Span::mixed_site()));
					handler_name
				});

				let args = quote_spanned! {paren.span.join().resolved_at(Span::mixed_site())=>
					(#self_: ::std::pin::Pin<&Self>#comma #event: ::#asteracea::lignin::web::Event)
				};
				quote_spanned!(fn_.span.resolved_at(Span::mixed_site())=> {
					impl #component_name {
						fn #handler_name #args #body
					}

					unsafe {
						//SAFETY: Defined with a compatible signature directly above.
						// `Pin` is transparent, `&Self` is compatible with `*const Self` for valid pointers.
						::std::mem::transmute(Self::#handler_name as fn(_, _))
					}
				})
			}
			Handler::Predefined(predefined) => {
				quote_spanned!(predefined.span().resolved_at(Span::mixed_site())=> {
					// Deny using component state, since this isn't evaluated more than once.
					let _: fn() = || {
						// Make sure the signature matches
						let _: &dyn ::#asteracea::__::CallbackHandler::<Self, ::#asteracea::lignin::web::Event, _> = &#predefined;
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

		let validate_mode = if let EventName::Known(name) = name {
			let const_name = Ident::new(
				&name.to_string(),
				name.span().resolved_at(Span::mixed_site()),
			);
			match mode {
				EventMode::None => {
					quote_spanned!(name.span().resolved_at(Span::mixed_site())=> {
						use ::#asteracea::__::lignin_schema::{EventInfo, YesNo};
						const #const_name: () = if <dyn ::#asteracea::__::lignin_schema::events::#name as EventInfo>::Bubbles::IS_YES {
							panic!("Expected one of keywords `bubble` or `capture`, as this event bubbles.")
						};
					})
				}
				EventMode::Capture(capture) => {
					let panic = quote_spanned! {capture.span.resolved_at(Span::mixed_site())=>
						const fn #const_name() {
							panic!("Keyword `capture` is not valid for this event; the event does not bubble.")
						}
						#const_name()
					};
					quote_spanned!(name.span().resolved_at(Span::mixed_site())=> {
						use ::#asteracea::__::lignin_schema::{EventInfo, YesNo};
						const #const_name: () = if !<dyn ::#asteracea::__::lignin_schema::events::#name as EventInfo>::Bubbles::IS_YES {
							#panic
						};
					})
				}
				EventMode::Bubble(bubble) => {
					let panic = quote_spanned! {bubble.span.resolved_at(Span::mixed_site())=>
						const fn #const_name() {
							panic!("Keyword `bubble` is not valid for this event; the event does not bubble.")
						}
						#const_name()
					};
					quote_spanned!(name.span().resolved_at(Span::mixed_site())=> {
						use ::#asteracea::__::lignin_schema::{EventInfo, YesNo};
						const #const_name: () = if !<dyn ::#asteracea::__::lignin_schema::events::#name as EventInfo>::Bubbles::IS_YES {
							#panic
						};
					})
				}
			}
			.pipe(Some)
		} else {
			None
		};
		let mode = match mode {
			EventMode::None => None,
			EventMode::Capture(capture) => Some(quote_spanned!(capture.span=> .with_capture(true))),
			EventMode::Bubble(bubble) => Some(quote_spanned!(bubble.span=> .with_capture(false))),
		};

		let validate_active = if let EventName::Known(name) = name {
			active.map(|active| {
				let const_name = Ident::new(
					&name.to_string(),
					name.span().resolved_at(Span::mixed_site()),
				);
				let panic = quote_spanned! {active.span.resolved_at(Span::mixed_site())=>
					const fn #const_name() {
						panic!("Keyword `active` is not valid for this event; the event is not cancellable.")
					}
					#const_name()
				};
				quote_spanned!(name.span().resolved_at(Span::mixed_site())=> {
					use ::#asteracea::__::lignin_schema::{EventInfo, YesNo};
						const #const_name: () = if !<dyn ::#asteracea::__::lignin_schema::events::#name as EventInfo>::Bubbles::IS_YES {
						#panic
					};
				})
			})
		} else {
			None
		};
		let active = active.map(|active| quote_spanned!(active.span=> .with_passive(false)));

		let once = once.map(|once| quote_spanned!(once.span=> .with_once(true)));

		quote_spanned!(on.span.resolved_at(Span::mixed_site())=> {
			#validate_active
			#validate_mode
			let registration = #self_.#registration_field_name.get_or_create(|| {
				::#asteracea::lignin::CallbackRegistration::<Self, fn(::#asteracea::lignin::web::Event)>::new(
					#self_,
					#handler,
				)
			});

			::#asteracea::lignin::EventBinding {
				name: #name,
				options: ::#asteracea::lignin::EventBindingOptions::new()#active #once #mode,
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
