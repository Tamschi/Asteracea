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

pub struct Argument {
	pub fn_arg: PatType,
	pub question: Option<Token![?]>,
	pub default: Option<(Token![=], Expr)>,
}

impl Parse for ConstructorArgument {
	fn parse(input: ParseStream) -> Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let capture = if input.peek(Token![priv]) {
			input.parse::<Token![priv]>().unwrap();
			Capture::Yes(Visibility::Inherited)
		} else {
			match input.parse()? {
				Visibility::Inherited => Capture::No,
				visibility => Capture::Yes(visibility),
			}
		};
		let pat;
		let question;
		let colon_token;
		let ty;
		unquote!(input, #pat #question #colon_token #ty);
		Ok(Self {
			argument: Argument {
				fn_arg: PatType {
					attrs,
					pat,
					colon_token,
					ty,
				},
				question,
				default: input.call(parse_default)?,
			},
			capture,
		})
	}
}

impl Parse for Argument {
	fn parse(input: ParseStream) -> Result<Self> {
		//TODO: This function makes a pretty good case for declaration and call syntax for quote.
		// Maybe something like `#let(pat)`, `#do(Attribute::parse_outer => attr)` and `#do let(parse_default => default)`.
		let attrs = input.call(Attribute::parse_outer)?;
		let pat;
		let question;
		let colon_token;
		let ty;
		unquote!(input, #pat #question #colon_token #ty);
		Ok(Self {
			fn_arg: PatType {
				attrs,
				pat,
				colon_token,
				ty,
			},
			question,
			default: input.call(parse_default)?,
		})
	}
}

fn parse_default(input: ParseStream) -> Result<Option<(Token![=], Expr)>> {
	input
		.parse::<Option<_>>()
		.unwrap()
		.map(|eq| Ok((eq, input.parse()?)))
		.transpose()
}
