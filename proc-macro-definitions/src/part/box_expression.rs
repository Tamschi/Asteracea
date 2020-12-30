use super::{CaptureDefinition, GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
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
	ConstParam, Error, ExprPath, GenericArgument, GenericParam, Generics, Ident, LifetimeDef,
	PathArguments, Result, Token, TypeGenerics, TypeParam, TypePath, Visibility, WhereClause,
};

#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct BoxExpression<C: Configuration> {
	box_: Token![box],
	vis: Either<Token![priv], Visibility>,
	field_name: Ident,
	type_: Option<(
		Token![:],
		Either<
			(
				Token![struct],
				Ident,
				Option<Token![::]>,
				Generics,
				Option<Token![;]>,
			),
			(
				ExprPath,
				// Must end with semicolon.
				Option<(WhereClause, Token![;])>,
			),
		>,
	)>,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for BoxExpression<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let box_: Token![box] = input.parse()?;

		let vis = if let Some(priv_) = input.parse().unwrap() {
			Some(Either::Left(priv_))
		} else {
			match input.parse().unwrap() {
				Visibility::Inherited => None,
				vis => Some(Either::Right(vis)),
			}
		};
		let (vis, field_name, type_) = if let Some(vis) = vis {
			let field_name = input.parse()?;

			let type_ = if let Some(colon) = input.parse().unwrap() {
				let type_ = if let Some(struct_) = input.parse().unwrap() {
					let name = input.parse()?;
					let double_colon: Option<Token![::]> = input.parse()?;
					let (generics, semicolon) = if double_colon.is_some() {
						let mut generics: Generics = input.parse()?;
						generics.where_clause = input.parse()?;
						let semicolon = if generics.where_clause.is_some() {
							Some(input.parse()?)
						} else {
							None
						};
						(generics, semicolon)
					} else {
						(Generics::default(), None)
					};
					Either::Left((struct_, name, double_colon, generics, semicolon))
				} else {
					let path = input.parse()?;
					let where_clause: Option<WhereClause> = input.parse()?;
					if let Some(where_clause) = where_clause.as_ref() {
						if let Some(last) = where_clause.predicates.pairs().last() {
							if last.punct().is_none() {
								return Err(Error::new_spanned(
									last,
									"Each `where`-predicate must end with a `,` here.",
								));
							}
						} else {
							return Err(Error::new_spanned(
								where_clause.where_token,
								"A `where` clause can't be empty here.",
							));
						}
					}
					let where_clause = where_clause
						.map(|w| Result::Ok((w, input.parse::<Token![;]>()?)))
						.transpose()?;
					Either::Right((path, where_clause))
				};
				Some((colon, type_))
			} else {
				None
			};

			(vis, field_name, type_)
		} else {
			(
				Either::Left(Token![priv](box_.span)),
				cx.storage_context.next_field(box_.span),
				None,
			)
		};

		fn strip_params(
			arguments: &Punctuated<GenericParam, Token![,]>,
		) -> Punctuated<GenericParam, Token![,]> {
			arguments
				.pairs()
				.map(|pair| {
					Pair::new(
						match pair.value() {
							GenericParam::Type(t) => GenericParam::Type(TypeParam {
								attrs: vec![],
								ident: t.ident.clone(),
								colon_token: None,
								bounds: Punctuated::default(),
								eq_token: None,
								default: None,
							}),
							GenericParam::Lifetime(l) => GenericParam::Lifetime(LifetimeDef {
								attrs: vec![],
								lifetime: l.lifetime.clone(),
								colon_token: None,
								bounds: Punctuated::default(),
							}),
							GenericParam::Const(c) => GenericParam::Type(TypeParam {
								attrs: vec![],
								ident: c.ident.clone(),
								colon_token: None,
								bounds: Punctuated::default(),
								eq_token: None,
								default: None,
							}),
						},
						pair.punct().cloned().cloned(),
					)
				})
				.collect()
		}

		fn generic_arguments_to_generic_params(
			arguments: &Punctuated<GenericArgument, Token![,]>,
		) -> Result<Punctuated<GenericParam, Token![,]>> {
			arguments
				.pairs()
				.map(|pair| {
					Ok(Pair::new(
						match pair.value() {
							syn::GenericArgument::Lifetime(l) => {
								GenericParam::Lifetime(LifetimeDef {
									attrs: vec![],
									lifetime: l.clone(),
									colon_token: None,
									bounds: Punctuated::default(),
								})
							}
							syn::GenericArgument::Type(t) => GenericParam::Type(TypeParam {
								attrs: vec![],
								ident: parse2(t.to_token_stream())?,
								colon_token: None,
								bounds: Punctuated::default(),
								eq_token: None,
								default: None,
							}),
							syn::GenericArgument::Binding(_) => {
								todo!("box type generic binding")
							}
							syn::GenericArgument::Constraint(_) => {
								todo!("box type generic constraint")
							}
							syn::GenericArgument::Const(_) => {
								todo!("box type generic const")
							}
						},
						pair.punct().cloned().cloned(),
					))
				})
				.collect()
		}

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
					(Cow::Borrowed(generics), true), // FIXME: This shouldn't actually generate a phantom! Removing it is a breaking change.
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

		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			generics.as_ref(),
		);
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let resolved_vis = match &vis {
			Either::Left(_) => Visibility::Inherited,
			Either::Right(vis) => vis.clone(),
		};

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

		let boxed_value = parse_context.storage_context.value(&type_path);

		call2_strict(
			quote_spanned! {box_.span=>
				|#resolved_vis #field_name: ::std::boxed::Box<#type_path> = {::std::boxed::Box::new(#boxed_value)}|;
			},
			|input| CaptureDefinition::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.unwrap()
		.debugless_unwrap_none();

		if let Some(generated_type_name) = generated_type_name {
			let type_definition = parse_context.storage_context.type_definition(
				cx.item_visibility,
				&generated_type_name,
				&generics,
			);

			cx.random_items
				.push(quote_spanned! {box_.span.resolved_at(Span::mixed_site())=>
					#[allow(non_camel_case_types)]
					#type_definition
				})
		}

		cx.random_items.extend(parse_context.random_items);

		Ok(Self {
			box_,
			vis,
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
				let #field_name = &*this.#field_name;
				let this = #field_name;
				#content
			}),
		)
	}
}
