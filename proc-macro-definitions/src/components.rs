use std::panic::{catch_unwind, AssertUnwindSafe};

use component::Component;
use loess::{
	parse_once, scaffold::SquareBrackets, Error, ErrorPriority, Errors, HandledPanic, Input,
	IntoTokens, PopFrom,
};
use proc_macro2::{Span, TokenStream};

pub fn components(input: TokenStream) -> TokenStream {
	let mut input = Input {
		tokens: input.into_iter().collect(),
		end: Span::call_site(),
	};

	let mut errors = Errors::new();

	let Ok(SquareBrackets { contents: root, .. }) = parse_once(&mut input, &mut errors) else {
		return errors.collect_tokens(&TokenStream::new());
	};

	let mut components: Vec<Component> = vec![];
	'panic_to_error: {
		match catch_unwind(AssertUnwindSafe(|| {
			while !input.is_empty() {
				let len = input.len();
				match Component::pop_from(&mut input, &mut errors) {
					Ok(component) | Err(Some(component)) => components.push(component),
					Err(None) => (),
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
	components.into_tokens(&root, &mut output);
	output
}

mod component;
