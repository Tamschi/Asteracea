use quote::ToTokens;
use syn::{
	parse::{Parse, ParseStream},
	Attribute, Expr, PatType, Result, Token, Visibility,
};
use unquote::unquote;

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
	pub fn_arg: PatType,
	pub question: Option<Token![?]>,
	pub default: Option<(Token![=], Expr)>,
}

impl Parse for ConstructorArgument {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote!(input,
			#do let Attributes::parse_outer => attrs
			#let capture
			#let pat
			#let question
			#let colon_token
			#let ty
			#do let Default::parse => default
		);
		Ok(Self {
			argument: Argument {
				fn_arg: PatType {
					attrs: attrs.into_inner(),
					pat,
					colon_token,
					ty,
				},
				question,
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
			#let pat
			#let question
			#let colon_token
			#let ty
			#do let Default::parse => default
		);
		Ok(Self {
			fn_arg: PatType {
				attrs: attrs.into_inner(),
				pat,
				colon_token,
				ty,
			},
			question,
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

struct Default(Option<(Token![=], Expr)>);
impl Default {
	fn into_inner(self) -> Option<(Token![=], Expr)> {
		self.0
	}
}
impl Parse for Default {
	fn parse(input: ParseStream) -> Result<Self> {
		input
			.parse::<Option<_>>()
			.unwrap()
			.map(|eq| Ok((eq, input.parse()?)))
			.transpose()
			.map(Self)
	}
}
impl ToTokens for Default {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if let Some((eq, expr)) = self.0.as_ref() {
			eq.to_tokens(tokens);
			expr.to_tokens(tokens);
		}
	}
}
