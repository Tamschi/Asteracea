use super::{GenerateContext, LetSelf, Part};
use crate::{
	asteracea_ident,
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::DebuglessUnwrap;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, Ident, Result, Token, Visibility};
use tap::Pipe;

pub mod kw {
	syn::custom_keyword!(bind);
}

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct Bind<C: Configuration> {
	bind: kw::bind,
	visibility: Visibility,
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	move_: Option<Token![move]>,
	binding_expression: TokenStream,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for Bind<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let bind: kw::bind = input.parse()?;
		let storage_configuration: StorageConfiguration = input.parse()?;

		let visibility = storage_configuration.visibility();

		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(bind.span));

		let type_configuration = storage_configuration.type_configuration();

		let nested_generics = type_configuration.generics()?;
		let auto_generics = nested_generics.is_none();
		let nested_generics = nested_generics.unwrap_or_else(|| cx.storage_generics.clone());

		let move_ = input.parse().unwrap();

		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			&nested_generics,
		);
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, cx.storage_generics)?;

		let binding_expression = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(bind.span);
		let node = quote_spanned!(bind.span=> node);
		call2_strict(
			quote_spanned! {bind.span.resolved_at(Span::mixed_site())=>
				let #visibility self.#field_name = pin ::#asteracea::try_lazy_init::LazyTransform::<
						#asteracea::__::rhizome::sync::NodeHandle<
							::core::any::TypeId,
							::core::any::TypeId,
							::#asteracea::__::rhizome::sync::DynValue,
						>,
						#type_path,
					>
					::new(#node.clone_handle());
			},
			|input| LetSelf::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.expect("bind storage let self");

		if type_configuration.type_is_generated() {
			cx.assorted_items.extend(
				type_configuration.struct_definition(
					vec![],
					visibility.clone(),
					type_path
						.path
						.segments
						.last()
						.expect("generated storage type last segment")
						.ident
						.clone(),
					&parse_context.storage_context,
					cx.storage_generics,
				)?,
			)
		}

		cx.assorted_items.extend(parse_context.assorted_items);

		Ok(Self {
			bind,
			visibility,
			field_name,
			type_configuration,
			move_,
			binding_expression,
			content,
		})
	}
}

impl<C: Configuration> Bind<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.bind.span);
		let field_name = &self.field_name;
		let field_pinned = Ident::new(&format!("{}_pinned", field_name), field_name.span());
		let node = quote_spanned!(self.bind.span=> node);
		let move_ = &self.move_;
		let binding_expression = &self.binding_expression;
		let content = self.content.part_tokens(cx)?;

		quote_spanned!(self.bind.span.resolved_at(Span::mixed_site())=> {
			let #field_name = this.#field_pinned();
			let #field_name = #field_name
				.get_or_create_or_poison(
					#move_ |#node| -> ::std::result::Result<_, ::#asteracea::error::Escalation> {
						Ok(#binding_expression)
					}
				)
				.map_err(|first_time_error| first_time_error.unwrap_or_else(|| todo!("construct repeat error")))?;
			let #field_name = unsafe {
				// SAFETY:
				// We already know the field itself is pinned properly, and the `LazyTransform` won't move its value around either.
				::std::pin::Pin::new_unchecked(#field_name)
			};
			let this = #field_name;
			#content
		})
		.pipe(Ok)
	}
}
