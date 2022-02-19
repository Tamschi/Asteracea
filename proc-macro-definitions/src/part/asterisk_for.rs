use super::{GenerateContext, Part};
use crate::{
	asteracea_ident,
	part::LetSelf,
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::DebuglessUnwrap;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	braced, parse::ParseStream, parse_quote_spanned, spanned::Spanned, token::Brace,
	visit_mut::VisitMut, Error, Expr, Ident, Label, Pat, Result, Token, Type, TypeReference,
};
use tap::Pipe;
use unquote::unquote;

mod kw {
	use syn::custom_keyword;
	custom_keyword!(keyed);
}

#[allow(dead_code)]
pub struct AsteriskFor<C: Configuration> {
	asterisk: Token![*],
	label: Option<Label>,
	for_: Token![for],
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	pat: Pat,
	in_: Token![in],
	iterable: Expr,
	brace: Brace,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for AsteriskFor<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let storage_configuration: StorageConfiguration;
		let for_: Token![for];
		unquote! {input,
			#let asterisk
			#let label
			#for_
			#storage_configuration
			#let pat
			#let in_
		};

		let iterable = Expr::parse_without_eager_brace(input)?;

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

		let content;
		let brace = braced!(content in input);

		let content = Box::new(Part::parse_required_with_context(
			&content,
			&mut parse_context,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, cx.storage_generics)?;

		let item_state = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(for_.span);
		let node = quote_spanned!(for_.span=> node);

		let item_state = quote_spanned!(for_.span.resolved_at(Span::mixed_site())=> asterisk_for.push(#item_state));
		let braced_item_state = quote_spanned!(brace.span=> { #item_state });

		call2_strict(
			quote_spanned! {for_.span.resolved_at(Span::mixed_site())=>
				let #visibility self.#field_name: ::core::pin::Pin<::std::boxed::Box::<[#type_path]>> = {
						let mut asterisk_for = ::std::vec::Vec::<#type_path>::new();
						#label #for_ #pat #in_ #iterable #braced_item_state
						asterisk_for.into_boxed_slice().into()
				};
			},
			|input| LetSelf::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.expect("for loop storage let self");

		if type_configuration.type_is_generated() {
			cx.assorted_items.extend(
				type_configuration.struct_definition(
					vec![],
					visibility,
					type_path
						.path
						.segments
						.last()
						.expect("`*for`: generated storage type last segment")
						.ident
						.clone(),
					&parse_context.storage_context,
					cx.storage_generics,
				)?,
			)
		}

		cx.assorted_items.extend(parse_context.assorted_items);

		Ok(Self {
			asterisk,
			label,
			for_,
			field_name,
			type_configuration,
			pat,
			in_,
			iterable,
			brace,
			content,
		})
	}
}

impl<C: Configuration> AsteriskFor<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.for_.span);
		let bump = Ident::new("bump", self.for_.span);

		let Self {
			label,
			asterisk,
			for_,
			type_configuration,
			field_name,
			pat,
			in_,
			iterable,
			brace,
			content,
		} = self;

		let for_span_mixed_site = for_.span.resolved_at(Span::mixed_site());

		let content = content.part_tokens(cx)?;
		let content = quote_spanned! {for_span_mixed_site=>
			let #field_name = unsafe { ::core::pin::Pin::new_unchecked(#field_name) };
			let this = #field_name;
			asterisk_for_items.push(#content)
		};
		let content = quote_spanned!(brace.span=> {
			#content
		});

		quote_spanned!(for_span_mixed_site=> {
			let asterisk_for = &this.#field_name;
			let asterisk_for = &**asterisk_for;
			let mut asterisk_for_items = ::#asteracea::bumpalo::vec![in #bump];
			#label #for_ #field_name in asterisk_for.iter() #content
			::#asteracea::lignin::Node::Multi(asterisk_for_items.into_bump_slice())
		})
		.pipe(Ok)
	}
}
