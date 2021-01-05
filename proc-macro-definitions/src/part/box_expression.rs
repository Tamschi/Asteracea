use super::{CaptureDefinition, GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use either::Either;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use std::borrow::Cow;
use syn::{
	parse::ParseStream,
	parse2,
	punctuated::{Pair, Punctuated},
	Error, ExprPath, GenericArgument, GenericParam, Generics, Ident, LifetimeDef, PathArguments,
	Result, Token, TypeParam, TypePath, Visibility,
};

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

		//OLD
		let (type_path, generated_type_name, (generics, add_phantom)): (
			ExprPath,
			Option<Ident>,
			(Cow<Generics>, bool),
		) = if let Some(type_) = type_.as_ref() {
			match &type_.1 {
				Either::Left((_, name, double_colon, generics, _semicolon)) => (
					{
						let generics = Generics {
							lt_token: generics.lt_token,
							params: strip_params(&generics.params),
							gt_token: generics.gt_token,
							where_clause: None,
						};
						parse2(quote_spanned!(Ident::span(name)=> #name#double_colon#generics))
							.unwrap()
					},
					Some(name.clone()),
					(Cow::Borrowed(generics), false),
				),
				Either::Right((path, where_clause)) => (
					ExprPath::clone(path),
					None,
					(
						Cow::Owned({
							let path: TypePath = parse2(path.to_token_stream())?;
							let arguments = &path.path.segments.last().unwrap().arguments;
							let generics = match arguments {
								PathArguments::None => Generics::default(),
								PathArguments::AngleBracketed(a_bra_args) => Generics {
									lt_token: Some(a_bra_args.lt_token),
									params: generic_arguments_to_generic_params(&a_bra_args.args)?,
									gt_token: Some(a_bra_args.gt_token),
									where_clause: where_clause.as_ref().map(|(w, _)| w).cloned()
								},
								PathArguments::Parenthesized(par_args) => {
									return Err(Error::new_spanned(par_args, "Parenthesized generic arguments are not supported in this position."))
								}
							};
							generics
						}),
						false,
					),
				),
			}
		} else {
			let type_name = cx.storage_context.generated_type_name(&field_name);
			(
				{
					let (_, type_generics, _) = cx.storage_generics.split_for_impl();
					let type_generics = type_generics.as_turbofish();
					parse2(quote_spanned!(type_name.span()=> #type_name#type_generics)).unwrap()
				},
				Some(type_name),
				(Cow::Borrowed(cx.storage_generics), true),
			)
		};
		//END OF OLD

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

		if add_phantom {
			let phantom_generics = generics
				.type_params()
				.map(|t| TypeParam {
					attrs: t.attrs.clone(),
					ident: t.ident.clone(),
					colon_token: None,
					bounds: Punctuated::default(),
					eq_token: None,
					default: None,
				})
				.collect::<Vec<_>>();
			call2_strict(
				quote_spanned! {box_.span.resolved_at(Span::mixed_site())=>
					|__Asteracea__phantom = ::std::marker::PhantomData::<(#(#phantom_generics, )*)>::default()|;
				},
				|input| {
					CaptureDefinition::<C>::parse_with_context(input, &mut parse_context)
						.unwrap()
						.debugless_unwrap_none()
				},
			)
			.unwrap()
		}

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

		if let Some(generated_type_name) = generated_type_name {
			let type_definition = parse_context.storage_context.type_definition(
				&[],
				cx.item_visibility,
				&generated_type_name,
				&generics,
			)?;

			cx.random_items
				.push(quote_spanned! {box_.span.resolved_at(Span::mixed_site())=>
					#[allow(non_camel_case_types)]
					#type_definition
				})
		}

		cx.random_items.extend(parse_context.random_items);

		Ok(Self {
			box_,
			visibility,
			field_name,
			type_,
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
