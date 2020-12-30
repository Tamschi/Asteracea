use super::{CaptureDefinition, GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use either::Either;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parse::ParseStream, parse2, ExprPath, Ident, Result, Token, Visibility};

#[allow(clippy::type_complexity)]
pub struct BoxExpression<C: Configuration>(
	Token![box],
	Option<(
		Either<Token![priv], Visibility>,
		Ident,
		Option<(Token![:], Either<(Token![struct], Ident), ExprPath>)>,
	)>,
	Box<Part<C>>,
);

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
		let binding = if let Some(vis) = vis.clone() {
			let name = input.parse()?;

			let type_ = if let Some(colon) = input.parse().unwrap() {
				let type_ = if let Some(struct_) = input.parse().unwrap() {
					Either::Left((struct_, input.parse()?))
				} else {
					Either::Right(input.parse()?)
				};
				Some((colon, type_))
			} else {
				None
			};

			Some((vis, name, type_))
		} else {
			None
		};

		let storage = &cx.storage;
		let (field_name, type_path): (Ident, ExprPath) = if let Some(binding) = &binding {
			let field_name = Clone::clone(&binding.1);
			let type_path = if let Some(type_) = &binding.2 {
				match &type_.1 {
					Either::Left((_, name)) => {
						parse2(quote_spanned!(Ident::span(name)=> #name)).unwrap()
					}
					Either::Right(path) => ExprPath::clone(path),
				}
			} else {
				let type_name = cx.storage_context.generated_type_name(&field_name);
				parse2(quote_spanned!(type_name.span() => #type_name)).unwrap()
			};
			(field_name, type_path)
		} else {
			let field_name = cx.storage_context.next_field(box_.span);
			let type_name = cx.storage_context.generated_type_name(&field_name);
			let type_path = parse2(quote_spanned!(type_name.span() => #type_name)).unwrap();
			(field_name, type_path)
		};
		let storage = parse2(quote_spanned!(box_.span=> #storage.#field_name)).unwrap();

		let mut parse_context =
			cx.new_nested(storage, cx.storage_context.generated_type_name(&field_name));
		let contents = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let resolved_vis = match &vis {
			Some(custom) => match custom {
				Either::Left(_) => Visibility::Inherited,
				Either::Right(vis) => vis.clone(),
			},
			None => Visibility::Inherited,
		};

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

		Ok(Self(box_, binding, contents))
	}
}

impl<C: Configuration> BoxExpression<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		self.2.part_tokens(cx)
	}
}
