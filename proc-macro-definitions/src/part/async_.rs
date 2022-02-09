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

//TODO?: Explicit moving into the render closure?
#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct Async<C: Configuration> {
	async_: Token![async],
	visibility: Visibility,
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for Async<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let async_: Token![async] = input.parse()?;
		let storage_configuration: StorageConfiguration = input.parse()?;

		let visibility = storage_configuration.visibility();

		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(async_.span));

		let type_configuration = storage_configuration.type_configuration();

		let nested_generics = type_configuration.generics()?;
		let auto_generics = nested_generics.is_none();
		let nested_generics = nested_generics.unwrap_or_else(|| cx.storage_generics.clone());

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

		let storage_value = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(async_.span);
		let node = quote_spanned!(async_.span.resolved_at(Span::call_site())=> node);
		call2_strict(
			quote_spanned! {async_.span=>
				let #visibility self.#field_name =
					pin ::#asteracea::include::async_::Async::<#type_path>
					::new(::std::boxed::Box::pin({
						let #node = ::std::sync::Arc::clone(&#node);
						async move { ::#asteracea::error::Result::Ok(#storage_value) }
					}));
			},
			|input| LetSelf::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.expect("box expression let self");

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
			async_,
			visibility,
			field_name,
			type_configuration,
			content,
		})
	}
}

impl<C: Configuration> Async<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let field_name = &self.field_name;
		let field_name_pinned = Ident::new(&format!("{}_pinned", field_name), field_name.span());
		let bump = quote_spanned!(self.async_.span.resolved_at(Span::call_site())=> bump);
		let content = self.content.part_tokens(cx)?;

		let asteracea = asteracea_ident(self.async_.span);
		Ok(
			quote_spanned! (self.async_.span.resolved_at(Span::mixed_site())=> {
				this.#field_name_pinned().as_async_content(Box::new(|#bump| {
					let this = this.#field_name_pinned().storage_pinned()?;
					let #field_name = this.as_ref();
					let this = #field_name;
					::#asteracea::error::Result::Ok(#content)
				}))
			}),
		)
	}
}
