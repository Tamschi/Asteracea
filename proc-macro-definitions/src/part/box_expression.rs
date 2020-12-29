use super::{GenerateContext, Part};
use crate::{
	parse_with_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use either::Either;
use proc_macro2::TokenStream;
use syn::{parse::ParseStream, ExprPath, Ident, Result, Token, Visibility};

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
		let box_ = input.parse()?;

		let vis = if let Ok(priv_) = input.parse() {
			Some(Either::Left(priv_))
		} else {
			match input.parse().unwrap() {
				Visibility::Inherited => None,
				vis => Some(Either::Right(vis)),
			}
		};
		let binding = if let Some(vis) = vis {
			let name = input.parse()?;

			let type_ = if let Ok(colon) = input.parse() {
				let type_ = if let Ok(struct_) = input.parse() {
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

		let mut parse_context = ParseContext {
			component_name: cx.component_name.clone(),
			..ParseContext::default()
		};
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
