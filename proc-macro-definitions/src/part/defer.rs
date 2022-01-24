use super::{
	BlockParentParameters, CaptureDefinition, GenerateContext, ParentParameterParser, Part,
};
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
use syn::{parse::ParseStream, Ident, Result, Visibility};
use tap::Pipe;

pub mod kw {
	syn::custom_keyword!(defer);
}

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct Defer<C: Configuration> {
	defer: kw::defer,
	visibility: Visibility,
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for Defer<C> {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
		parent_parameter_parser: &mut dyn ParentParameterParser,
	) -> Result<Self::Output> {
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

		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			&nested_generics,
		);
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
			parent_parameter_parser,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, cx.storage_generics)?;

		let deferred_value = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(defer.span);
		let node = quote_spanned!(defer.span=> node);
		call2_strict(
			quote_spanned! {defer.span.resolved_at(Span::mixed_site())=>
				pin |
					#visibility #field_name =
						::#asteracea::storage::Defer::<'static, #type_path>
						::new(::std::boxed::Box::new({
							#[allow(unused_variables)]
							let #node = #node.clone_handle();
							//FIXME: This shouldn't `move` everything.
							move || {
								let #node = &*#node;
								Ok(#deferred_value)
							}
						}))
				|;
			},
			|input| {
				CaptureDefinition::<C>::parse_with_context(input, cx, &mut BlockParentParameters)
			},
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
			content,
		})
	}
}

impl<C: Configuration> Defer<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let field_name = &self.field_name;
		let field_pinned = Ident::new(&format!("{}_pinned", field_name), field_name.span());
		let content = self.content.part_tokens(cx)?;

		quote_spanned!(self.defer.span.resolved_at(Span::mixed_site())=> {
			let #field_name = this.#field_pinned().get_or_poison()?;
			let this = #field_name;
			#content
		})
		.pipe(Ok)
	}
}
