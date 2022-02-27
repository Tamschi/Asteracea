use super::{GenerateContext, Part};
use crate::{
	asteracea_ident,
	part::LetSelf,
	storage_configuration::{StorageConfiguration, StorageTypeConfiguration},
	storage_context::{ParseContext, ParseWithContext},
	workaround_module::Configuration,
};
use call2_for_syn::call2_strict;
use debugless_unwrap::DebuglessUnwrap;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	braced, parse::ParseStream, parse_quote_spanned, spanned::Spanned, token::Brace,
	visit_mut::VisitMut, Error, Expr, Ident, Pat, Result, Token, Type, TypeReference,
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
	type_: Option<(Token![:], Type)>,
	key: Option<(kw::keyed, Expr)>,
	key_type: Option<(Token![=>], Type)>,
	in_: Token![in],
	iterable: Expr,
	brace: Brace,
	content: Box<Part<C>>,
}

impl<C: Configuration> ParseWithContext for For<C> {
	type Output = Self;

	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output> {
		let storage_configuration: StorageConfiguration;
		let for_: Token![for];
		unquote! {input,
			#for_
			#storage_configuration
			#let pat
		};

		let type_ = input
			.parse::<Option<Token![:]>>()
			.expect("infallible")
			.map(|colon| Ok::<_, Error>((colon, input.parse()?)))
			.transpose()?;

		let key = input
			.parse::<Option<kw::keyed>>()
			.expect("infallible")
			.map(|colon| Ok::<_, Error>((colon, input.parse()?)))
			.transpose()?;

		let key_type: Option<(_, Type)> = input
			.parse::<Option<Token![=>]>>()
			.expect("infallible")
			.map(|colon| Ok::<_, Error>((colon, input.parse()?)))
			.transpose()?;

		unquote! {input,
			#let in_
		};

		let iterable = Expr::parse_without_eager_brace(input)?;

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

		let content;
		let brace = braced!(content in input);

		let content = Box::new(Part::parse_required_with_context(
			&content,
			&mut parse_context,
		)?);

		let type_path =
			type_configuration.type_path(&cx.storage_context, &field_name, cx.storage_generics)?;

		let manufactured_item_state = parse_context.storage_context.value(
			type_configuration.type_is_generated(),
			&type_path,
			auto_generics,
		);

		let asteracea = asteracea_ident(for_.span);

		let k = if let Some((_, key_type)) = &key_type {
			Some(key_type.to_token_stream())
		} else {
			type_.as_ref().map(|(colon, type_): &(_, Type)| {
				let type_ = make_type_static(type_.clone());
				quote_spanned! {colon.span.resolved_at(Span::mixed_site())=>
					<<#type_ as ::#asteracea::__::UnBorrow>::Target as ::std::borrow::ToOwned>::Owned
				}
			})
		}
		.into_iter();
		call2_strict(
			quote_spanned! {for_.span.resolved_at(Span::mixed_site())=>
				let #visibility self.#field_name = ::core::cell::RefCell::<::#asteracea::include::For::<'static, #type_path#(, #k)*>>::new(
					::#asteracea::include::For::new({
						#[allow(unused_variables)]
						let resource_node = resource_node.clone_handle();
						move || Ok(#manufactured_item_state)
					})
				);
			},
			|input| LetSelf::<C>::parse_with_context(input, cx),
		)
		.debugless_unwrap()
		.expect("for loop storage let self");

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
			type_,
			key,
			key_type,
			in_,
			iterable,
			brace,
			content,
		})
	}
}

fn make_type_static(mut type_: Type) -> Type {
	struct StaticInserter;
	impl VisitMut for StaticInserter {
		fn visit_type_reference_mut(&mut self, i: &mut TypeReference) {
			i.lifetime =
				Some(parse_quote_spanned!(i.span().resolved_at(Span::mixed_site())=> 'static));
			syn::visit_mut::visit_type_reference_mut(self, i)
		}
	}

	StaticInserter.visit_type_mut(&mut type_);
	type_
}

impl<C: Configuration> For<C> {
	pub fn part_tokens(&self, cx: &GenerateContext) -> Result<TokenStream> {
		let asteracea = asteracea_ident(self.for_.span);
		let bump = Ident::new("bump", self.for_.span);

		let Self {
			for_,
			field_name,
			pat,
			type_,
			key,
			key_type,
			iterable,
			brace,
			content,
			..
		} = self;

		let for_span_mixed_site = for_.span.resolved_at(Span::mixed_site());

		let selector = match (type_, key, key_type) {
			(None, Some((keyed, key)), None) => {
				quote_spanned! {keyed.span.resolved_at(Span::mixed_site())=>
					// There is really nothing to go on here. Use dynamically typed keys.
					|#pat| ::core::result::Result::Ok(
						::#asteracea::include::__for_::InferredQ::from_ref(#key)
					)
				}
			}
			(None, None, None) => quote_spanned! {for_span_mixed_site=>
				// There is really nothing to go on here. Use dynamically typed keys.
				|item: &mut _| ::core::result::Result::Ok(
					::#asteracea::include::__for_::InferredQ::from_ref(
						::#asteracea::__::UnBorrow::one_borrow(item)
					)
				)
			},
			(_, Some((keyed, key)), _) => {
				quote_spanned! {keyed.span.resolved_at(Span::mixed_site())=>
					|#pat| ::core::result::Result::Ok(#key)
				}
			}
			(Some(_), _, _) => quote_spanned! {for_span_mixed_site=>
				// There is a key type, so infer/derive forwards towards Q:
				|item: &mut _| ::core::result::Result::Ok(::#asteracea::__::UnBorrow::one_borrow(item))
			},
			(None, _, Some(_)) => quote_spanned! {for_span_mixed_site=>
				//Help Wanted: This is necessary to resolve e.g. `for i => u8 in &[1, 2, 3, 4, 5]` "backwards",
				// but is there a broader way to do that?
				|item: &mut _| ::core::result::Result::Ok(::core::ops::Deref::deref(item))
			},
		};

		let generics = type_.as_ref().map(|(colon, type_)| {
			quote_spanned! {colon.span.resolved_at(Span::mixed_site())=>
				::<_, <#type_ as ::#asteracea::__::UnBorrow>::Target, _, _>
			}
		});

		let item_type = type_.as_ref().map(|(colon, type_)| {
			quote_spanned! {for_span_mixed_site=>
				#colon (#type_, _)
			}
		});

		let content = content.part_tokens(cx)?;
		let content = quote_spanned! {for_span_mixed_site=>
			let (#pat, reorderable_storage)#item_type = item?;
			let #field_name = reorderable_storage.as_ref().storage();
			let this = #field_name;
			for_items.push(::#asteracea::lignin::ReorderableFragment {
				dom_key: reorderable_storage.dom_key,
				content: #content,
			})
		};
		let content = quote_spanned!(brace.span=> {
			#content
		});

		quote_spanned!(for_span_mixed_site=> {
			let mut for_ = ::core::cell::RefCell::borrow_mut(&this.#field_name);
			let for_ = &mut *for_;
			let sequence = for_.__Asteracea__reproject_try_by#generics(
				#iterable,
				#selector,
			);
			let mut for_items = ::#asteracea::bumpalo::vec![in #bump];
			for item in sequence #content
			::#asteracea::lignin::Node::Keyed(for_items.into_bump_slice())
		})
		.pipe(Ok)
	}
}
