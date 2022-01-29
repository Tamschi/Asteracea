use super::{GenerateContext, Part};
use crate::{
	asteracea_ident,
	part::{BlockParentParameters, CaptureDefinition},
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::ParseWithContext,
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{Expr, Ident, Pat, Result, Token};
use tap::Pipe;
use unquote::unquote;

mod kw {
	use syn::custom_keyword;
	custom_keyword!(keyed);
}

#[allow(dead_code)]
pub struct For<C: Configuration> {
	for_: Token![for],
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	pat: Pat,
	keyed: kw::keyed,
	key: Expr,
	in_: Token![in],
	iterable: Expr,
	comma: Token![,],
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for For<C> {
	type Output = Self;

	fn parse_with_context(
		input: syn::parse::ParseStream<'_>,
		cx: &mut crate::storage_context::ParseContext,
		parent_parameter_parser: &mut dyn super::ParentParameterParser,
	) -> Result<Self::Output> {
		//TODO: Very broken, refactor this into `Part` in general and just have these preface any part.
		parent_parameter_parser.parse_any(input, cx)?;

		let storage_configuration: StorageConfiguration;
		let for_: Token![for];
		unquote! {input,
			#for_
			#storage_configuration
			#let pat
			#let keyed
			#let key
			#let in_
			#let iterable
			#let comma
		};

		let visibility = storage_configuration.visibility();
		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(for_.span));
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

		let manufactured_item_state = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(for_.span);
		let node = quote_spanned!(for_.span=> node);
		call2_strict(
			quote_spanned! {for_.span.resolved_at(Span::mixed_site())=>
				pin |
					#visibility #field_name =
						::#asteracea::storage::For::<'static, #type_path>
						::new({
							#[allow(unused_variables)]
							let #node = ::std::sync::Arc::clone(&#node);
							move |_| Ok(#manufactured_item_state)
						})
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
					visibility,
					type_path
						.path
						.segments
						.last()
						.expect("`for`: generated storage type last segment")
						.ident
						.clone(),
					&parse_context.storage_context,
					cx.storage_generics,
				)?,
			)
		}

		cx.assorted_items.extend(parse_context.assorted_items);

		Ok(Self {
			for_,
			field_name,
			type_configuration,
			pat,
			keyed,
			key,
			in_,
			iterable,
			comma,
			content,
		})
	}
}

impl<C: Configuration> For<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.for_.span);
		let bump = Ident::new("bump", self.for_.span);

		let field_name = &self.field_name;
		let field_pinned = Ident::new(&format!("{}_pinned", field_name), field_name.span());

		quote_spanned!(self.for_.span.resolved_at(Span::mixed_site())=> {
			let for_items = ::#asteracea::bumpalo::vec![in #bump];
		})
		.pipe(Ok)
	}
}
