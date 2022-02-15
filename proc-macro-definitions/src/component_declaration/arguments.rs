use crate::asteracea_ident;
use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse_quote_spanned,
	spanned::Spanned,
	Attribute, Expr, FnArg, PatType, Result, Token, Visibility,
};
use tap::Pipe;
use unquote::unquote;

pub struct ConstructorArgument {
	pub capture: Capture,
	pub injection_dyn: Option<Token![dyn]>,
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
			#let injection_dyn
			#let pat
			#let question
			#let colon_token
			#let ty
			#do let DefaultParameter::parse => default
		);
		Ok(Self {
			capture,
			injection_dyn,
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
		})
	}
}

impl Parse for Argument {
	fn parse(input: ParseStream) -> Result<Self> {
		let attrs = Attributes::parse_outer(input)?;
		if let Some(dot2) = input.parse::<Option<Token![..]>>().expect("infallible") {
			// This is a content argument.
			// For now, only a very minimal feature is supported.
			let bump = quote_spanned! (dot2.span()=>
				'bump
			);
			let asteracea = asteracea_ident(dot2.span());
			Self {
				fn_arg: match parse_quote_spanned! {dot2.span().resolved_at(Span::mixed_site())=>
					__Asteracea__anonymous_content: (
						::#asteracea::__::AnonymousContentParentParameters,
						::std::boxed::Box::<
							dyn '_ + ::core::ops::FnOnce(&#bump ::#asteracea::bumpalo::Bump) -> ::std::result::Result::<
								::#asteracea::lignin::Guard::<
									#bump,
									::#asteracea::lignin::ThreadSafe,
								>,
								::#asteracea::error::Escalation,
							>
						>,
					)
				} {
					FnArg::Receiver(_) => unreachable!(),
					FnArg::Typed(pat_type) => pat_type,
				},
				question: None,
				default: None,
			}
		} else {
			unquote!(input,
				#let pat
				#let question
				#let colon_token
				#let ty
				#do let DefaultParameter::parse => default
			);
			Self {
				fn_arg: PatType {
					attrs: attrs.into_inner(),
					pat,
					colon_token,
					ty,
				},
				question,
				default: default.into_inner(),
			}
		}
		.pipe(Ok)
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
