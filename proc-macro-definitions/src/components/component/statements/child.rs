use loess::{grammar, Error, ErrorPriority, Errors, Input, PeekFrom, PopFrom, SimpleSpanned};
use loess_rust::{Await, CurlyBraces, Dot, Identifier, Parentheses, Semi, SquareBrackets};
use proc_macro2::TokenStream;

use super::{Statement, Storage};

grammar! {
	pub struct Child: PeekFrom, PopFrom, IntoTokens {
		pub identifier: ChildIdentifier,
		pub new_args: Option<Parentheses>,
		pub dot_await: Option<DotAwait>,
		pub render_args: Option<SquareBrackets>,
		pub storage: Option<Storage>,
		pub children: ChildChildren,
	}

	pub struct DotAwait: PeekFrom, PopFrom, IntoTokens {
		pub dot: Dot,
		pub r#await: Await,
	}

	pub enum ChildIdentifier: IntoTokens {
		Local(Identifier),
		Substrate(Identifier),
		Qualified(TokenStream),
	} else "Expected child identifier.";

	pub enum ChildChildren: PopFrom, IntoTokens {
		Void(Semi),
		Braces(CurlyBraces<Vec<Statement>>), //TODO: Named slots.
	} else "Expected `;` or `{`.";
}

impl PeekFrom for ChildIdentifier {
	fn peek_from(input: &Input) -> bool {
		//TODO: Or ColonColon.
		Identifier::peek_from(input)
	}
}

impl PopFrom for ChildIdentifier {
	fn pop_from(input: &mut Input, errors: &mut Errors) -> Result<Self, ()> {
		Ok(
			if let Some(identifier) = Identifier::peek_pop_from(input, errors)? {
				let c = identifier
					.0
					.to_string()
					.chars()
					.next()
					.expect("No zero-length identifiers, hopefully!");
				if c.is_uppercase() {
					Self::Local(identifier)
				} else if c.is_lowercase() {
					Self::Substrate(identifier)
				} else {
					return Err(errors.push(Error::new(
						ErrorPriority::GRAMMAR,
						"Expected identifier to be either upper- or lowercase.",
						[identifier.span()],
					)));
				}
			} else {
				return Err(errors.push(Error::new(
					ErrorPriority::GRAMMAR,
					"Expected child type identifier or path statement.",
					[input.front_span()],
				)));
			},
		)
	}
}
