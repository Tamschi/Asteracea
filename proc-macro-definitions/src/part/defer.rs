use super::{CaptureDefinition, GenerateContext, Part};
use crate::{
	asteracea_ident,
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, Ident, Result, Token, Visibility};
use wyz::Pipe;

pub mod kw {
	//TODO: Split this into `lazy` and `init_once` expressions!
	syn::custom_keyword!(defer);
}

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct Defer<C: Configuration> {
	defer: kw::defer,
	visibility: Visibility,
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	dynamicism: Option<(Token![dyn], Option<Token![move]>)>,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for Defer<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let defer: kw::defer = input.parse()?;
		let storage_configuration: StorageConfiguration = input.parse()?;

		let visibility = storage_configuration.visibility();

		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(defer.span));

		let type_configuration = storage_configuration.type_configuration();

		let nested_generics = type_configuration.generics()?;
		let auto_generics = nested_generics.is_none();
		let nested_generics = nested_generics.unwrap_or_else(|| cx.storage_generics.clone());

		let dynamicism: Option<Token![dyn]> = input.parse()?;
		let dynamicism = dynamicism
			.map(|dyn_| Result::Ok((dyn_, input.parse()?)))
			.transpose()?;

		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			&nested_generics,
		);
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, &cx.storage_generics)?;

		let deferred_value = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(defer.span);
		call2_strict(
			if dynamicism.is_some() {
				todo!("defer dynamicism")
			} else {
				quote_spanned! {defer.span=>
					pin |
						#visibility #field_name: ::#asteracea::lazy_init::LazyTransform<::std::boxed::Box<dyn FnOnce() -> ::std::result::Result<#type_path, ::#asteracea::error::ExtractableResolutionError>>, #type_path> =
							{::#asteracea::lazy_init::LazyTransform::new(::std::boxed::Box::new(move || Ok(#deferred_value)))}
					|;
				}
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
			defer,
			visibility,
			field_name,
			type_configuration,
			dynamicism,
			content,
		})
	}
}

impl<C: Configuration> Defer<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.defer.span);
		let field_name = &self.field_name;
		let field_pinned = Ident::new(&format!("{}_pinned", field_name), field_name.span());
		let content = self.content.part_tokens(cx)?;

		if let Some((_, move_)) = &self.dynamicism {
			todo!("defer dynamicism")
		} else {
			quote_spanned!(self.defer.span.resolved_at(Span::mixed_site())=> {
				let #field_name = this.#field_pinned();
				let #field_name = #field_name.try_get_or_create(|init| init())?;
				let #field_name = unsafe {
					// SAFETY:
					// We already know the field itself is pinned properly, and the `LazyTransform` won't move its value around either.
					::std::pin::Pin::new_unchecked(#field_name)
				};
				let this = #field_name;
				#content
			})
		}
		.pipe(Ok)
	}
}
