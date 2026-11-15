use std::ops::ControlFlow::Continue;

use loess::{
	Errors, Input, IntoTokens, parse_once, quote_into_mixed_site, scaffold::SquareBrackets,
};
use proc_macro2::{Group, Ident, Punct, Span, TokenStream, TokenTree};

pub fn extract_dot_use(input: TokenStream) -> TokenStream {
	let mut input = Input {
		tokens: input.into_iter().collect(),
		end: Span::call_site(),
	};

	let mut errors = Errors::new();

	let Continue(Some(SquareBrackets { contents: root, .. })) = parse_once(&mut input, &mut errors)
	else {
		return errors.collect_tokens(&TokenStream::new());
	};

	let has_move = has_move(input.tokens.clone());
	let mut extracted = vec![];
	let input = extract_dot_use_(input.tokens.into_iter().collect(), &mut extracted);
	if extracted.is_empty() {
		input
	} else {
		let mut output = errors.collect_tokens(&root);
		quote_into_mixed_site!(Span::mixed_site(), &root, &mut output, {
			{
				{#for (ident, dot, r#use) in extracted {
					{#located_at(r#use.span()) {
						let {#(&ident)}
						{#located_at(dot.span()) {
							=
							{#located_at(r#use.span()) {
								{#root}::ToOwned::to_owned({#(ident)})
							}}
							;
						}}
					}}
				}}
				{#if !has_move {
					//TODO: There's definitely a better solution to moving the upvars.
					//TODO: Probably by assigning them to themselves at the start.
					move
				}} {#(input)}
			}
		});
		output
	}
}

fn has_move(tts: impl IntoIterator<Item = TokenTree>) -> bool {
	match tts.into_iter().next() {
		Some(TokenTree::Group(group)) => has_move(group.stream().into_iter()),
		Some(TokenTree::Ident(ident)) if ident.to_string() == "move" => true,
		_ => false,
	}
}

fn extract_dot_use_(input: TokenStream, extracted: &mut Vec<(Ident, Punct, Ident)>) -> TokenStream {
	let mut input: Vec<_> = input.into_iter().collect();
	let mut i = 0;
	while i < input.len() {
		if let (
			Some(TokenTree::Ident(ident)),
			Some(TokenTree::Punct(dot)),
			Some(TokenTree::Ident(r#use)),
		) = (input.get(i), input.get(i + 1), input.get(i + 2))
			&& dot.as_char() == '.'
			&& r#use.to_string() == "use"
		{
			let ident = ident.clone();
			let mut dot_use = input.drain((i + 1)..=(i + 2));
			let Some(TokenTree::Punct(dot)) = dot_use.next() else {
				unreachable!()
			};
			let Some(TokenTree::Ident(r#use)) = dot_use.next() else {
				unreachable!()
			};
			assert_eq!(dot_use.count(), 0);
			extracted.push((ident, dot, r#use));
			continue;
		}
		if let Some(TokenTree::Group(group)) = input.get_mut(i) {
			let span = group.span();
			*group = Group::new(
				group.delimiter(),
				extract_dot_use_(group.stream(), extracted),
			);
			group.set_span(span);
		}
		i += 1;
	}
	input.into_iter().collect()
}
