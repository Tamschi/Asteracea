use quote::quote;
use syn::{
	parse::{Parse, ParseStream},
	Attribute, Error, FnArg, PatType, Result, Token, Visibility,
};

pub struct ConstructorArgument {
	pub capture: Capture,
	pub fn_arg: PatType,
}

pub enum Capture {
	No,
	//TODO: It's possible to redefine these quick captures in the constructor body right now,
	// but that's potentially confusing since the data dependency order jumps down and back up.
	// Assign captured parameters immediately to binding of the same name but mixed_site resolution to prevent manipulation.
	// Types that are Copy will still be usable in the constructor regardless, and for anything else there are more explicit captures.
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
