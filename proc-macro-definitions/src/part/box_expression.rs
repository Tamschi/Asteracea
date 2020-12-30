use super::{GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
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
		let binding = if let Some(vis) = vis {
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
		let field_name: Ident = if let Some(binding) = &binding {
			Clone::clone(&binding.1)
		} else {
			cx.storage_context.next_field(box_.span)
		};
		let storage = parse2(quote_spanned!(box_.span=> #storage.#field_name)).unwrap();

		let mut parse_context = cx.new_nested(storage);
		let contents = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
		)?);

		//TODO: Capture

		Ok(Self(box_, binding, contents))
	}
}

impl<C: Configuration> BoxExpression<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		self.2.part_tokens(cx)
	}
}
