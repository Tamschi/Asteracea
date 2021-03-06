use crate::{
	component_declaration::FieldDefinition,
	storage_context::{ParseContext, ParseWithContext},
};
use core::marker::PhantomData;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
	braced, parenthesized,
	parse::{ParseStream, Result},
	punctuated, AttrStyle, Attribute, Error, Ident, Path, Token, Type,
};
use take_mut::take;

pub struct CaptureDefinition<C> {
	access: TokenStream,
	_phantom: PhantomData<C>,
}

pub mod kw {
	syn::custom_keyword!(pin);
}

impl<C> ParseWithContext for CaptureDefinition<C> {
	type Output = Option<Self>;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let mut attributes = input.call(Attribute::parse_outer)?;

		let pin: Option<kw::pin> = input.parse()?;

		input.parse::<Token![|]>()?;

		attributes.append(
			&mut input
				.call(Attribute::parse_inner)?
				.into_iter()
				.map(|inner| Attribute {
					style: AttrStyle::Outer,
					..inner
				})
				.collect(),
		);
		attributes.append(&mut input.call(Attribute::parse_outer)?);

		let visibility = input.parse()?;
		let name: Ident = input.parse()?;

		let field_type;
		let initial_value;

		let shorthand_lookahead = input.lookahead1();
		if shorthand_lookahead.peek(Token![:]) {
			// Long form
			input.parse::<Token![:]>()?;
			field_type = {
				let field_type = input.call(Type::without_plus)?;
				quote!(#field_type)
			};

			let initial_value_buffer;
			input.parse::<Token![=]>()?;
			let brace = braced!(initial_value_buffer in input);
			let initial_value_tokens: TokenStream = initial_value_buffer.parse()?;
			let brace_span = proc_macro::Span::mixed_site()
				.located_at(brace.span.unstable())
				.into();
			initial_value = quote_spanned! (brace_span=> {#initial_value_tokens});
		} else if shorthand_lookahead.peek(Token![=]) {
			// Shorthand
			input.parse::<Token![=]>()?;

			// Supporting ExprPath here would be better, but considerably more complicated.
			let path: Path = input.parse()?;
			if path.segments.len() < 2 {
				return Err(Error::new_spanned(
					path,
					"Expected qualified path to constructor.",
				));
			}

			let type_path_colon = path.leading_colon;

			let segments: Vec<_> = path.segments.into_pairs().collect();
			let (constructor_name, type_path_segments) = segments.split_last().unwrap();

			let mut type_path_segments: Vec<_> = type_path_segments.to_vec();

			let mut constructor_punct = Default::default();
			{
				let last_i = type_path_segments.len() - 1;
				take(
					&mut type_path_segments[last_i],
					|last_in_type| match last_in_type {
						punctuated::Pair::Punctuated(a_type, punct) => {
							constructor_punct = punct;
							punctuated::Pair::End(a_type)
						}
						_ => unreachable!(),
					},
				);
			}

			if let punctuated::Pair::Punctuated(_, trailing) = constructor_name {
				return Err(Error::new_spanned(
					trailing,
					"Expected path ending with constructor name.",
				));
			}

			field_type = quote! {
				#type_path_colon#(#type_path_segments)*
			};

			//TODO: This is a bit hacky with regards to chaining and error escalation.
			let parameters;
			let paren = parenthesized!(parameters in input);
			let parameters: TokenStream = parameters.parse()?;
			let question: Option<Token![?]> = input.parse().ok();
			initial_value = quote_spanned! {paren.span=>
				#field_type#constructor_punct#constructor_name(#parameters)#question
			}
		} else {
			return Err(shorthand_lookahead.error());
		}

		input.parse::<Token![|]>()?;

		let field_definition = FieldDefinition {
			attributes,
			visibility,
			name: name.clone(),
			field_type,
			initial_value,
			structurally_pinned: pin.is_some(),
		};
		cx.storage_context.push(field_definition);

		let access = {
			if input.peek(Token![;]) {
				input.parse::<Token![;]>()?;
				None
			} else {
				Some(if let Some(pin) = pin {
					let pinned_name = Ident::new(&format!("{}_pinned", name), name.span());
					let pin_parens = quote_spanned!(pin.span=> ());
					quote_spanned!(pinned_name.span().resolved_at(Span::mixed_site())=> this.#pinned_name#pin_parens)
				} else {
					quote_spanned!(name.span().resolved_at(Span::mixed_site())=> this.#name)
				})
			}
		};

		Ok(access.map(|access| Self {
			access,
			_phantom: PhantomData,
		}))
	}
}

impl<C> ToTokens for CaptureDefinition<C> {
	fn to_tokens(&self, output: &mut TokenStream) {
		self.access.to_tokens(output);
	}
}
