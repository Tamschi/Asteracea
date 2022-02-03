use super::ParentParameterParser;
use crate::{
	component_declaration::FieldDefinition,
	storage_context::{ParseContext, ParseWithContext},
};
use core::marker::PhantomData;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{ParseStream, Result},
	punctuated::Pair,
	AttrStyle, Attribute, Error, Expr, Ident, Path, Token, Type, TypePath, TypeTuple,
};
use tap::Pipe;

pub struct LetSelf<C> {
	access: TokenStream,
	_phantom: PhantomData<C>,
}

pub mod kw {
	syn::custom_keyword!(pin);
}

/// This one is a bit unusual: It always returns an access expression, but the caller-accessible code path through [`super::Part`] always drops it.
/// The access expression is only used by other expressions that partially lower into these bindings.
impl<C> ParseWithContext for LetSelf<C> {
	type Output = Self;
	fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
		_: &mut dyn ParentParameterParser,
	) -> Result<Self::Output> {
		let mut attributes = input.call(Attribute::parse_outer)?;

		let _let_: Token![let] = input.parse()?;
		let visibility = input.parse()?;
		let _self_: Token![self] = input.parse()?;
		let _dot: Token![.] = input.parse()?;
		let name: Ident = input.parse()?;

		let explicit_type: Option<(Token![:], Type)> = input
			.parse::<Option<Token![:]>>()
			.expect("infallible")
			.map(|colon| Ok::<_, Error>((colon, input.call(Type::without_plus)?)))
			.transpose()?;

		let _eq: Token![=] = input.parse()?;

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

		let pin: Option<kw::pin> = input.parse()?;
		let initial_value: Expr = input.parse()?;
		let _semicolon: Token![;] = input.parse()?;

		let type_ = if let Some((_, type_)) = explicit_type {
			type_
		} else {
			guesstimate_type(&initial_value)?
		};

		let field_definition = FieldDefinition {
			attributes,
			visibility,
			name: name.clone(),
			field_type: type_,
			initial_value: initial_value.into_token_stream(),
			structurally_pinned: pin.is_some(),
		};
		cx.storage_context.push(field_definition);

		let access = if let Some(pin) = pin {
			let pinned_name = Ident::new(&format!("{}_pinned", name), name.span());
			let pin_parens = quote_spanned!(pin.span=> ());
			quote_spanned!(pinned_name.span().resolved_at(Span::mixed_site())=> this.#pinned_name#pin_parens)
		} else {
			quote_spanned!(name.span().resolved_at(Span::mixed_site())=> this.#name)
		};

		Ok(Self {
			access,
			_phantom: PhantomData,
		})
	}
}

fn guesstimate_type(value: &Expr) -> Result<Type> {
	let mut expr = value;
	let type_: Type = loop {
		expr = match expr {
					    Expr::Await(await_) => &await_.base,
					    Expr::Call(call) => &call.func,
					    Expr::Cast(cast) => break (*cast.ty).clone(),
					    Expr::Group(group) => &group.expr,
					    Expr::Paren(paren) => &paren.expr,
					    Expr::Path(path) =>  break {
							let mut segments = path.path.segments.clone();
							segments.pop().filter(|_| !segments.is_empty() || path.qself.is_some()).ok_or_else(|| Error::new_spanned(expr, "This path must have at least two segments to guesstimate a field type."))?;
							if let Some(final_segment) = segments.pop() {
								segments.push_value(final_segment.into_value())
							}
							Type::Path(TypePath {
								qself: path.qself.clone(),
								path: Path { leading_colon: path.path.leading_colon, segments }
							})
						},
					    Expr::Struct(struct_) => break Type::Path(TypePath { qself: None, path: struct_.path.clone() }),
					    Expr::Try(try_) => &try_.expr,
					    Expr::Tuple(tuple) => break Type::Tuple(TypeTuple {
							paren_token: tuple.paren_token,
							elems: tuple
								.elems
								.pairs()
								.map(|pair| {
									Ok(Pair::new(
										guesstimate_type(pair.value())?,
										pair.punct().copied().copied(),
									))
								})
								.collect::<Result<_>>()?,
						}),
					    Expr::Type(type_) => break (*type_.ty).clone(),
					    other => return Err(Error::new_spanned(other, "Could not guesstimate field type. Try a simpler expression or set the field type explicitly."))
				    }.pipe(|box_| &**box_);
	};
	Ok(type_)
}

impl<C> ToTokens for LetSelf<C> {
	fn to_tokens(&self, output: &mut TokenStream) {
		self.access.to_tokens(output);
	}
}
