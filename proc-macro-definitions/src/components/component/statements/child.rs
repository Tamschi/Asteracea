use loess::{
	grammar,
	scaffold::{CurlyBraces, Parentheses, SquareBrackets},
	Error, ErrorPriority, Errors, Input, PeekFrom, PopFrom, PopParsedFrom, SimpleSpanned,
};
use loess_rust::{
	ident::Identifier,
	lex::{
		keywords::Await,
		token::punct::{Dot, Semi},
	},
};
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

impl PopParsedFrom for ChildIdentifier {
	type Parsed = Self;

	fn pop_parsed_from(
		input: &mut Input,
		errors: &mut Errors,
	) -> Result<Self::Parsed, Option<Self::Parsed>> {
		if let Some(identifier) = Identifier::peek_pop_from(input, errors).map_err(|_| None)? {
			let c = identifier
				.0
				.to_string()
				.chars()
				.next()
				.expect("No zero-length identifiers, hopefully!");
			if c.is_uppercase() {
				Ok(Self::Local(identifier))
			} else if c.is_lowercase() {
				Ok(Self::Substrate(identifier))
			} else {
				errors.push(Error::new(
					ErrorPriority::GRAMMAR,
					"Expected identifier to be either upper- or lowercase.",
					[identifier.span()],
				));
				Err(None)
			}
		} else {
			errors.push(Error::new(
				ErrorPriority::GRAMMAR,
				"Expected child type identifier or path statement.",
				[input.front_span()],
			));
			Err(None)
		}
	}
}
