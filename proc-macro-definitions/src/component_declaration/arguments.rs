use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse2, Attribute, Expr, Ident, PatType, Result, Token, Type, Visibility,
};
use unquote::unquote;
use wyz::Pipe;

use crate::asteracea_ident;

pub struct ConstructorArgument {
	pub capture: Capture,
	pub argument: Argument,
}

pub enum Capture {
	No,
	//TODO: It's possible to redefine these quick captures in the constructor body right now,
	// but that's potentially confusing since the data dependency order jumps down and back up.
	// Assign captured parameters immediately to binding of the same name but mixed_site resolution to prevent manipulation.
	// Types that are Copy will still be usable in the constructor regardless, and for anything else there are more explicit captures.
	Yes(syn::Visibility),
}
impl Parse for Capture {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if input.peek(Token![priv]) {
			input.parse::<Token![priv]>().unwrap();
			Capture::Yes(Visibility::Inherited)
		} else {
			match input.parse()? {
				Visibility::Inherited => Capture::No,
				visibility => Capture::Yes(visibility),
			}
		})
	}
}
impl ToTokens for Capture {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Capture::No => (),
			Capture::Yes(visibility) => visibility.to_tokens(tokens),
		}
	}
}

pub struct Argument {
	pub item_name: Option<(Ident, Token![/])>,
	pub fn_arg: PatType,
	pub repeat_mode: RepeatMode,
	pub optional: Option<Token![?]>,
	pub default: Option<(Token![=], Expr)>,
}
impl Argument {
	pub fn effective_type(&self) -> Type {
		effective_type(
			self.fn_arg.ty.as_ref().clone(),
			self.repeat_mode,
			self.optional,
		)
	}
}

pub fn effective_type(ty: Type, repeat_mode: RepeatMode, optional: Option<Token![?]>) -> Type {
	match repeat_mode {
		RepeatMode::Single => ty,
		RepeatMode::AtLeastOne(token) => {
			let asteracea = asteracea_ident(token.span);
			parse2(quote_spanned!(token.span=> ::#asteracea::vec1::Vec1<#ty>))
				.expect("parameter helper definitions at-least-one type")
		}
		RepeatMode::AnyNumber(token) => parse2(quote_spanned!(token.span=> ::std::vec::Vec<#ty>))
			.expect("parameter helper definitions any-number type"),
	}
	.pipe(|ty| {
		if let Some(question) = optional {
			parse2(quote_spanned!(question.span=> ::core::option::Option<#ty>))
				.expect("parameter helper definitions optional type")
		} else {
			ty
		}
	})
}

impl Parse for ConstructorArgument {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote!(input,
			#do let Attributes::parse_outer => attrs
			#let capture
			#do let ItemName::parse => item_name
			#let pat
			#let repeat_mode
			#let optional
			#let colon_token
			#let ty
			#do let DefaultParameter::parse => default
		);
		Ok(Self {
			argument: Argument {
				item_name: item_name.into_inner(),
				fn_arg: PatType {
					attrs: attrs.into_inner(),
					pat,
					colon_token,
					ty,
				},
				repeat_mode,
				optional,
				default: default.into_inner(),
			},
			capture,
		})
	}
}

impl Parse for Argument {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote!(input,
			#do let Attributes::parse_outer => attrs
			#do let ItemName::parse => item_name
			#let pat
			#let repeat_mode
			#let optional
			#let colon_token
			#let ty
			#do let DefaultParameter::parse => default
		);
		Ok(Self {
			item_name: item_name.into_inner(),
			fn_arg: PatType {
				attrs: attrs.into_inner(),
				pat,
				colon_token,
				ty,
			},
			repeat_mode,
			optional,
			default: default.into_inner(),
		})
	}
}

struct Attributes(Vec<Attribute>);
impl Attributes {
	fn parse_outer(input: ParseStream) -> Result<Self> {
		Attribute::parse_outer(input).map(Self)
	}
	fn into_inner(self) -> Vec<Attribute> {
		self.0
	}
}
impl ToTokens for Attributes {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for attr in self.0.iter() {
			attr.to_tokens(tokens)
		}
	}
}

struct ItemName(Option<(Ident, Token![/])>);
impl ItemName {
	pub fn into_inner(self) -> Option<(Ident, Token![/])> {
		self.0
	}
}
impl Parse for ItemName {
	fn parse(input: ParseStream) -> Result<Self> {
		input
			.peek2(Token![/])
			.then(|| Result::Ok((input.parse()?, input.parse()?)))
			.transpose()?
			.pipe(Self)
			.pipe(Ok)
	}
}
impl ToTokens for ItemName {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if let Some(item_name) = &self.0 {
			item_name.0.to_tokens(tokens);
			item_name.1.to_tokens(tokens);
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
	Single,
	AtLeastOne(Token![+]),
	AnyNumber(Token![*]),
}
impl Parse for RepeatMode {
	fn parse(input: ParseStream) -> Result<Self> {
		if let Some(plus) = input.parse().unwrap() {
			Self::AtLeastOne(plus)
		} else if let Some(asterisk) = input.parse().unwrap() {
			Self::AnyNumber(asterisk)
		} else {
			Self::Single
		}
		.pipe(Ok)
	}
}
impl ToTokens for RepeatMode {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Single => (),
			Self::AtLeastOne(plus) => plus.to_tokens(tokens),
			Self::AnyNumber(asterisk) => asterisk.to_tokens(tokens),
		}
	}
}

struct DefaultParameter(Option<(Token![=], Expr)>);
impl DefaultParameter {
	fn into_inner(self) -> Option<(Token![=], Expr)> {
		self.0
	}
}
impl Parse for DefaultParameter {
	fn parse(input: ParseStream) -> Result<Self> {
		input
			.parse::<Option<_>>()
			.unwrap()
			.map(|eq| Ok((eq, input.parse()?)))
			.transpose()
			.map(Self)
	}
}
impl ToTokens for DefaultParameter {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if let Some((eq, expr)) = self.0.as_ref() {
			eq.to_tokens(tokens);
			expr.to_tokens(tokens);
		}
	}
}
