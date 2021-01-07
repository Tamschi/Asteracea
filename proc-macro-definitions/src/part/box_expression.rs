use super::{CaptureDefinition, GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, Ident, Item, Result, Token, Visibility};

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct BoxExpression<C: Configuration> {
	box_: Token![box],
	visibility: Visibility,
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for BoxExpression<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let box_: Token![box] = input.parse()?;
		let storage_configuration: StorageConfiguration = input.parse()?;

		let visibility = storage_configuration.visibility();

		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(box_.span));

		let type_configuration = storage_configuration.type_configuration();

		let nested_generics = type_configuration
			.generics()?
			.unwrap_or_else(|| cx.storage_generics.clone());
		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			&nested_generics,
		);
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let type_path = type_configuration.type_path(&cx.storage_context, &field_name)?;

		let boxed_value = parse_context
			.storage_context
			.value(type_configuration.type_is_generated(), &type_path);

		call2_strict(
			quote_spanned! {box_.span=>
				|#visibility #field_name: ::std::pin::Pin<::std::boxed::Box<#type_path>> = {::std::boxed::Box::pin(#boxed_value)}|;
			},
			|input| CaptureDefinition::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.unwrap()
		.debugless_unwrap_none();

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
			box_,
			visibility,
			field_name,
			type_configuration,
			content,
		})
	}
}

impl<C: Configuration> BoxExpression<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let field_name = &self.field_name;
		let content = self.content.part_tokens(cx)?;

		Ok(
			quote_spanned! (self.box_.span.resolved_at(Span::mixed_site())=> {
				let #field_name = this.#field_name.as_ref();
				let this = #field_name;
				#content
			}),
		)
	}
}
