use super::{GenerateContext, Part};
use crate::{storage_context::ParseWithContext, workaround_module::Configuration};
use proc_macro2::TokenStream;
use syn::{Expr, Pat, Result, Token};
use unquote::unquote;

mod kw {
	use syn::custom_keyword;
	custom_keyword!(keyed);
}

pub struct For<C: Configuration> {
	for_: Token![for],
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

		unquote! {input,
			#let for_
			#let pat
			#let keyed
			#let key
			#let in_
			#let iterable
			#let comma
		};
		let content = Box::new(Part::parse_required_with_context(
			input,
			cx,
			parent_parameter_parser,
		)?);
		Ok(Self {
			for_,
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
		todo!("for tokens")
	}
}
