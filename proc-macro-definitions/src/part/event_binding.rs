use super::capture_definition::CaptureDefinition;
use crate::{
	asteracea_crate,
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
	braced,
	parse::ParseStream,
	parse_quote,
	token::{Add, Move},
	visit_mut::{self, VisitMut},
	Expr, ExprPath, ForeignItemFn, Ident, ItemFn, LitStr, Result,
};

pub struct EventBindingDefinition {
	prefix: Add,
	name: LitStr,
	field_name: Ident,
}

impl EventBindingDefinition {
	pub fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
	) -> Result<EventBindingDefinition> {
		let prefix: Add = input.parse()?;
		let name: LitStr = input.parse()?;
		let move_token: Option<Move> = input.parse().ok();
		let handler_body;
		let brace = braced!(handler_body in input);
		let handler_body: TokenStream = handler_body.parse()?;

		struct ReplaceSelf;
		impl VisitMut for ReplaceSelf {
			fn visit_expr_mut(&mut self, node: &mut Expr) {
				if let Expr::Path(ExprPath { path, .. }) = node {
					if path.leading_colon.is_none() && path.segments.len() == 1 {
						let segment = path.segments.first().unwrap();
						if segment.arguments.is_empty() {
							let ident = &segment.ident;
							if format!("{}", ident).starts_with("asteracea__") {
								//TODO: Handle this more gracefully and also check other custom identifiers.
								panic!("User-provided identifier starting with asteracea__ found in event handler")
							}
							if format!("{}", ident) == "self" {
								let replacement = Ident::new("asteracea__self", ident.span());
								*node = parse_quote!(#replacement);
								return;
							}
						}
					}
				}
				visit_mut::visit_expr_mut(self, node)
			}

			// Ignore function definitions, since they can redeclare `self` and an outer `self` isn't valid there.
			fn visit_foreign_item_fn_mut(&mut self, _: &mut ForeignItemFn) {}
			fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {}
		}
		let handler = quote_spanned! (brace.span=> { #handler_body });
		let mut handler = parse_quote!(#handler);
		ReplaceSelf.visit_expr_block_mut(&mut handler);

		let component_name = cx
			.component_name
			.as_ref()
			.expect("Component name not set in ParseContext");
		let handler = quote_spanned! {brace.span =>
			#move_token |#[allow(non_snake_case)] asteracea__self| {
				#[allow(non_snake_case)]
				let asteracea__self = asteracea__self
					.downcast_ref::<#component_name>()
					.expect(
						concat!(
							"Failed to downcast reference to component ",
							stringify!(#component_name)
						)
					);
				#handler
			}
		};

		let field_name = cx.storage_context.next_field(name.span());

		call2_strict(
			quote_spanned! {prefix.span=>
				#[allow(non_snake_case)] // This currently has no effect, hence `allow_non_snake_case_on_structure_workaround`.
				|#field_name: ::std::pin::Pin<::std::rc::Rc<dyn ::std::ops::Fn(&dyn ::core::any::Any)>> = { ::std::rc::Rc::pin(#handler) }|;
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
			prefix,
			name,
			field_name,
		})
	}

	pub fn part_tokens(&self) -> TokenStream {
		let EventBindingDefinition {
			prefix,
			name,
			field_name,
		} = self;
		let asteracea = asteracea_crate();
		let self_ident = Ident::new("self", prefix.span);

		quote_spanned! {name.span()=>
			#asteracea::lignin::EventBinding {
				name: #name,
				context: #asteracea::unsound_extend_reference(self.get_ref()),
				handler: #self_ident.#field_name.clone(),
			}
		}
	}
}
