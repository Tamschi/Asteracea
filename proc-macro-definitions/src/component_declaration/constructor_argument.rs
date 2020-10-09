use quote::quote;
use syn::{
	parse::Parse, parse::ParseStream, Attribute, Error, FnArg, PatType, Result, Token, Visibility,
};

pub struct ConstructorArgument {
	pub capture: Capture,
	pub fn_arg: PatType,
}

pub enum Capture {
	No,
	Yes(syn::Visibility),
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
		Ok(Self {
			fn_arg: match input.parse::<FnArg>()? {
				FnArg::Receiver(r) => {
					return Err(Error::new_spanned(
						r,
						"Component constructors cannot expect `self` parameters.",
					));
				}
				FnArg::Typed(pat_type)
					if matches!(capture, Capture::No) || pat_type.attrs.is_empty() =>
				{
					PatType { attrs, ..pat_type }
				}
				FnArg::Typed(PatType { attrs, .. }) => {
					return Err(Error::new_spanned(quote!(#(#attrs)*), "Attributes are currently not available in this position. Place them before the visibility modifier instead."));
				}
			},
			capture,
		})
	}
}
