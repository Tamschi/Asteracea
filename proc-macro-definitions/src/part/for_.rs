use super::{GenerateContext, Part};
use crate::{
	asteracea_ident,
	part::{BlockParentParameters, CaptureDefinition},
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::{DebuglessUnwrap, DebuglessUnwrapNone};
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	Error, Expr, FnArg, Ident, Pat, PatType, Result, Token, Type,
};
use tap::Pipe;
use unquote::unquote;

mod kw {
	use syn::custom_keyword;
	custom_keyword!(keyed);
}

#[allow(dead_code)]
pub struct For<C: Configuration> {
	for_: Token![for],
	field_name: Ident,
	type_configuration: StorageTypeConfiguration,
	pat: Pat,
	colon: Token![:],
	type_: Type,
	selector: Option<Selector>,
	in_: Token![in],
	iterable: Expr,
	comma: Token![,],
	content: Box<Part<C>>,
}

struct Selector {
	keyed: kw::keyed,
	key: Expr,
	key_type: Option<(Token![=>], Type)>,
}

impl Parse for Selector {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote! {input,
			#let keyed
			#let key
		}
		Ok(Self {
			keyed,
			key,
			key_type: input
				.peek(Token![=>])
				.then(|| Ok::<_, Error>((input.parse()?, input.parse()?)))
				.transpose()?,
		})
	}
}

impl<C: Configuration> ParseWithContext for For<C> {
	type Output = Self;

	fn parse_with_context(
		input: ParseStream<'_>,
		cx: &mut ParseContext,
		parent_parameter_parser: &mut dyn super::ParentParameterParser,
	) -> Result<Self::Output> {
		//TODO: Very broken, refactor this into `Part` in general and just have these preface any part.
		parent_parameter_parser.parse_any(input, cx)?;

		let storage_configuration: StorageConfiguration;
		let for_: Token![for];
		unquote! {input,
			#for_
			#storage_configuration
			#let pat_type
		};

		let (pat, colon, type_) = match pat_type {
			FnArg::Receiver(receiver) => {
				return Err(Error::new_spanned(receiver, "Expected `pat: Type`."))
			}
			FnArg::Typed(PatType {
				attrs,
				pat,
				colon_token,
				ty,
			}) if attrs.is_empty() => (*pat, colon_token, *ty),
			FnArg::Typed(PatType { attrs, .. }) => {
				return Err(Error::new_spanned(
					attrs
						.into_iter()
						.flat_map(ToTokens::into_token_stream)
						.collect::<TokenStream>(),
					"Unexpected attributes: No attributes expected.",
				))
			}
		};

		let selector: Option<Selector> =
			input.peek(kw::keyed).then(|| input.parse()).transpose()?;

		unquote! {input,
			#let in_
			#let iterable
			#let comma
		};

		let visibility = storage_configuration.visibility();
		let field_name = storage_configuration
			.field_name()
			.cloned()
			.unwrap_or_else(|| cx.storage_context.next_field(for_.span));
		let type_configuration = storage_configuration.type_configuration();
		let nested_generics = type_configuration.generics()?;
		let auto_generics = nested_generics.is_none();
		let nested_generics = nested_generics.unwrap_or_else(|| cx.storage_generics.clone());

		let mut parse_context = cx.new_nested(
			cx.storage_context.generated_type_name(&field_name),
			&nested_generics,
		);

		let content = Box::new(Part::parse_required_with_context(
			input,
			&mut parse_context,
			parent_parameter_parser,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, cx.storage_generics)?;

		let manufactured_item_state = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(for_.span);
		let node = quote_spanned!(for_.span=> node);

		let k = if let Some(selector) = &selector {
			selector.key_type.as_ref().map(|(_, key_type)| key_type)
		} else {
			Some(&type_)
		}.into_iter();
		call2_strict(
			quote_spanned! {for_.span.resolved_at(Span::mixed_site())=>
				|
					#visibility #field_name =
						::core::cell::RefCell::<::#asteracea::storage::For::<'static, #type_path#(, #k)*>>::new(
							::#asteracea::storage::For::new({
								#[allow(unused_variables)]
								let #node = ::std::sync::Arc::clone(&#node);
								move || Ok(#manufactured_item_state)
							})
						)
				|;
			},
			|input| {
				CaptureDefinition::<C>::parse_with_context(input, cx, &mut BlockParentParameters)
			},
		)
		.debugless_unwrap()
		.unwrap()
		.debugless_unwrap_none();

		if type_configuration.type_is_generated() {
			cx.assorted_items.extend(
				type_configuration.struct_definition(
					vec![],
					visibility,
					type_path
						.path
						.segments
						.last()
						.expect("`for`: generated storage type last segment")
						.ident
						.clone(),
					&parse_context.storage_context,
					cx.storage_generics,
				)?,
			)
		}

		cx.assorted_items.extend(parse_context.assorted_items);

		Ok(Self {
			for_,
			field_name,
			type_configuration,
			pat,
			colon,
			type_,
			selector,
			in_,
			iterable,
			comma,
			content,
		})
	}
}

impl<C: Configuration> For<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.for_.span);
		let bump = Ident::new("bump", self.for_.span);

		let Self {
			for_,
			field_name,
			pat,
			selector,
			iterable,
			content,
			..
		} = self;

		let for_ = for_.span.resolved_at(Span::mixed_site());

		let selector = if let Some(Selector { keyed, key, .. }) = selector {
			quote_spanned! {keyed.span.resolved_at(Span::mixed_site())=>
				|#pat| ::core::result::Result::Ok(#key)
			}
		} else {
			quote_spanned! {for_=>
				|item: &mut _| ::core::result::Result::Ok(&*item)
			}
		};

		let content = content.part_tokens(cx)?;

		quote_spanned!(for_=> {
			let mut for_ = ::core::cell::RefCell::borrow_mut(&this.#field_name);
			let for_ = &mut *for_;
			let sequence = for_.__Asteracea__update_try_by(
				#iterable,
				#selector,
			);
			let mut for_items = ::#asteracea::bumpalo::vec![in #bump];
			for item in sequence {
				let (#pat, #field_name) = item?;
				let this = #field_name;
				for_items.push(#content)
			}
			::#asteracea::lignin::Node::Multi(for_items.into_bump_slice())
		})
		.pipe(Ok)
	}
}
