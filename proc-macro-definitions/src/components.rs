use std::{
	ops::ControlFlow::{Break, Continue},
	panic::{catch_unwind, AssertUnwindSafe},
};

use component::Component;
use loess::{
	parse_once, scaffold::SquareBrackets, Error, ErrorPriority, Errors, HandledPanic, Input,
	IntoTokens, PopFrom,
};
use loess_rust::lex::token::punct::{Eq, Not, Pound};
use proc_macro2::{Group, Span, TokenStream};

use crate::lex::Substrate;

pub fn components(input: TokenStream) -> TokenStream {
	let mut input = Input {
		tokens: input.into_iter().collect(),
		end: Span::call_site(),
	};

	let mut errors = Errors::new();

	let Continue(Some(SquareBrackets { contents: root, .. })) = parse_once(&mut input, &mut errors)
	else {
		return errors.collect_tokens(&TokenStream::new());
	};

	let substrate = if let Continue(Some((
		Pound { .. },
		Not { .. },
		SquareBrackets {
			contents: (Substrate(kw), Eq { .. }, substrate),
			..
		},
	))) = parse_once(&mut input, &mut errors)
	{
		// Wrapping this here *might* result in better errors somewhere.
		let mut group = Group::new(proc_macro2::Delimiter::None, substrate);
		group.set_span(kw.span());
		group
	} else {
		return errors.collect_tokens(&TokenStream::new());
	};

	let mut components: Vec<Component> = vec![];
	'panic_to_error: {
		match catch_unwind(AssertUnwindSafe(|| {
			while !input.is_empty() {
				let len = input.len();
				match Component::pop_from(&mut input, &mut errors) {
					//TODO: Revise! Stop if misaligned?
					Continue(Some(component)) | Break(Some(component)) => {
						components.push(component)
					}
					Continue(None) | Break(None) => (),
				}
				if input.len() == len {
					input.tokens.pop_front().expect("unreachable");
				}
			}
		})) {
			Ok(()) => (),
			Err(panic) => errors.push(Error::new(
				ErrorPriority::PANIC,
				&format!(
					"proc macro panicked: {:?}",
					if panic.as_ref().is::<HandledPanic>() {
						break 'panic_to_error;
					} else if let Some(message) = panic.as_ref().downcast_ref::<String>() {
						message.clone()
					} else if let Some(message) = panic.as_ref().downcast_ref::<&'static str>() {
						message.to_string()
					} else {
						errors.push(Error::new(
							ErrorPriority::PANIC,
							"proc macro panicked",
							[input.front_span()],
						));
						break 'panic_to_error;
					}
				),
				[input.front_span()],
			)),
		}
	}

	let mut output = errors.collect_tokens(&root);
	for component in components {
		component.emit(&root, &mut output, &substrate);
	}
	output
}

mod component;
