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
use syn::{parse::ParseStream, parse2, ExprPath, Ident, Result, Token, Visibility};

#[allow(clippy::type_complexity)]
pub struct BoxExpression<C: Configuration> {
	box_: Token![box],
	vis: Either<Token![priv], Visibility>,
	field_name: Ident,
	type_: Option<(Token![:], Either<(Token![struct], Ident), ExprPath>)>,
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
					Either::Left((struct_, input.parse()?))
				} else {
					Either::Right(input.parse()?)
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

		let type_path: ExprPath = {
			let type_path = if let Some(type_) = type_.as_ref() {
				match &type_.1 {
					Either::Left((_, name)) => {
						parse2(quote_spanned!(Ident::span(name)=> #name)).unwrap()
					}
					Either::Right(path) => ExprPath::clone(path),
				}
			} else {
				let type_name = cx.storage_context.generated_type_name(&field_name);
				parse2(type_name.to_token_stream()).unwrap()
			};
			type_path
		};

		let mut parse_context = cx.new_nested(cx.storage_context.generated_type_name(&field_name));
		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		let resolved_vis = match &vis {
			Either::Left(_) => Visibility::Inherited,
			Either::Right(vis) => vis.clone(),
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
				let this = &*this.#field_name;
				let #field_name = this;
				#content
			}),
		)
	}
}
