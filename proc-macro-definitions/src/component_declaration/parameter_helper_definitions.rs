use crate::syn_ext::*;
use proc_macro2::Span;
use std::{iter, mem};
use syn::{
	parse_quote,
	punctuated::Punctuated,
	spanned::Spanned as _,
	token::{Brace, Paren},
	AngleBracketedGenericArguments, Attribute, Binding, Constraint, Expr, Field, FieldsNamed,
	GenericArgument, GenericParam, Generics, Ident, Lifetime, LifetimeDef,
	ParenthesizedGenericArguments, Path, PathArguments, PathSegment, ReturnType, Token, TraitBound,
	Type, TypeArray, TypeGroup, TypeParam, TypeParamBound, TypeParen, TypePath, TypeReference,
	TypeSlice, TypeTraitObject, TypeTuple, Visibility,
};
use wyz::Tap as _;

fn transform_lifetime(
	existing_lifetime: &mut Lifetime,
	lifetime: &Lifetime,
	adjust_lifetimes: bool,
) {
	if adjust_lifetimes && existing_lifetime.ident == "_" {
		*existing_lifetime = lifetime.clone();
	}
}

fn transform_path_segments<'a>(
	segments: impl IntoIterator<Item = &'a mut PathSegment>,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
	adjust_lifetimes: bool,
) {
	for segment in segments {
		match &mut segment.arguments {
			PathArguments::None => (),
			PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
				for arg in args.iter_mut() {
					match arg {
						GenericArgument::Lifetime(l) => {
							if adjust_lifetimes && l.ident == "_" {
								*l = lifetime.clone()
							}
						}
						GenericArgument::Type(ty)
						| GenericArgument::Binding(Binding { ty, .. }) => {
							transform_type(ty, lifetime, impl_generics, adjust_lifetimes)
						}
						GenericArgument::Constraint(Constraint { bounds, .. }) => {
							transform_type_param_bounds(
								bounds.iter_mut(),
								lifetime,
								impl_generics,
								adjust_lifetimes,
							)
						}
						GenericArgument::Const(_) => (
							// Do nothing and hope for the best.
						),
					}
				}
			}
			PathArguments::Parenthesized(ParenthesizedGenericArguments {
				inputs, output, ..
			}) => {
				// Don't modify lifetimes in callable's signatures.
				for input in inputs.iter_mut() {
					transform_type(input, lifetime, impl_generics, false)
				}
				match output {
					ReturnType::Default => (),
					ReturnType::Type(_, ty) => {
						transform_type(&mut *ty, lifetime, impl_generics, false)
					}
				}
			}
		}
	}
}

fn transform_type_param_bounds<'a>(
	bounds: impl IntoIterator<Item = &'a mut TypeParamBound>,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
	adjust_lifetimes: bool,
) {
	for b in bounds {
		match b {
			TypeParamBound::Trait(TraitBound {
				lifetimes, path, ..
			}) => {
				if let Some(l) = lifetimes.as_mut() {
					for l in l.lifetimes.iter_mut() {
						for l in l.bounds.iter_mut() {
							transform_lifetime(l, lifetime, adjust_lifetimes)
						}
					}
				};
				transform_path_segments(
					path.segments.iter_mut(),
					lifetime,
					impl_generics,
					adjust_lifetimes,
				)
			}
			TypeParamBound::Lifetime(l) => transform_lifetime(l, lifetime, adjust_lifetimes),
		}
	}
}

fn transform_type(
	ty: &mut Type,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
	adjust_lifetimes: bool,
) {
	#[allow(clippy::wildcard_in_or_patterns)]
	match ty {
		Type::Array(TypeArray { elem, .. })
		| Type::Paren(TypeParen { elem, .. })
		| Type::Group(TypeGroup { elem, .. })
		| Type::Slice(TypeSlice { elem, .. }) => {
			transform_type(elem, lifetime, impl_generics, adjust_lifetimes)
		}
		Type::BareFn(_) => todo!("Type::BareFn"),
		ty @ Type::ImplTrait(_) => {
			let impl_span = match ty {
				Type::ImplTrait(it) => it.impl_token.span(),
				_ => unreachable!(),
			};

			let impl_ident = Ident::new(&format!("IMPL_{}", impl_generics.len()), impl_span);

			let impl_trait = match mem::replace(
				ty,
				Type::Path(TypePath {
					qself: None,
					path: Path {
						leading_colon: None,
						segments: iter::once(PathSegment {
							ident: impl_ident.clone(),
							arguments: PathArguments::None,
						})
						.collect(),
					},
				}),
			) {
				Type::ImplTrait(it) => it,
				_ => unreachable!(),
			};

			let mut bounds = impl_trait.bounds;

			let applied_lifetime = transform_type_param_bounds(
				bounds.iter_mut(),
				lifetime,
				impl_generics,
				adjust_lifetimes,
			);

			impl_generics.push(TypeParam {
				attrs: vec![],
				ident: impl_ident,
				colon_token: Some(<Token![:]>::default()),
				bounds,
				eq_token: None,
				default: None,
			});

			applied_lifetime
		}
		Type::Infer(_) => todo!("Type::Infer"),
		Type::Never(_) => todo!("Type::Never"),
		Type::Path(TypePath { qself, path }) => {
			if let Some(qself) = qself.as_mut() {
				transform_type(&mut *qself.ty, lifetime, impl_generics, adjust_lifetimes)
			};
			transform_path_segments(
				path.segments.iter_mut(),
				lifetime,
				impl_generics,
				adjust_lifetimes,
			)
		}
		Type::Ptr(_) => (
			// Not actually allowed, but will error correctly, I think.
		),
		Type::Reference(TypeReference {
			lifetime: l, elem, ..
		}) => {
			if adjust_lifetimes && l.as_ref().map_or(true, |l| l.ident == "_") {
				*l = Some(lifetime.clone());
			}
			transform_type(&mut *elem, lifetime, impl_generics, adjust_lifetimes)
		}
		Type::TraitObject(TypeTraitObject { bounds, .. }) => transform_type_param_bounds(
			bounds.iter_mut(),
			lifetime,
			impl_generics,
			adjust_lifetimes,
		),
		Type::Tuple(TypeTuple { elems, .. }) => {
			for elem in elems {
				transform_type(elem, lifetime, impl_generics, adjust_lifetimes)
			}
		}
		Type::Verbatim(_) | Type::Macro(_) => (
			// Do nothing and hope for the best.
		),
		_ => {
			//TODO: Warn about unhandled type gammar if possible.
		}
	}
}

fn transform_generic_param(generic_param: &mut GenericParam, lifetime: &Lifetime) {
	let mut impl_generics = vec![];
	match generic_param {
		GenericParam::Type(type_param) => transform_type_param_bounds(
			type_param.bounds.iter_mut(),
			lifetime,
			&mut impl_generics,
			true,
		),
		GenericParam::Lifetime(lifetime_def) => {
			transform_lifetime(&mut lifetime_def.lifetime, lifetime, true);
			for bound in lifetime_def.bounds.iter_mut() {
				transform_lifetime(bound, lifetime, true)
			}
		}
		GenericParam::Const(_) => (),
	}
	assert!(impl_generics.is_empty())
}

#[derive(Debug)]
pub struct ParameterHelperDefintions {
	pub on_parameter_struct: Generics,
	pub parameter_struct_body: FieldsNamed,
	pub on_function: Generics,
	pub for_function_args: AngleBracketedGenericArguments,
	pub on_builder_function: Generics,
	pub for_builder_function_return: AngleBracketedGenericArguments,
}

#[derive(Debug, Copy, Clone)]
pub struct CustomArgument<'a> {
	pub attrs: &'a [Attribute],
	pub ident: &'a Ident,
	pub ty: &'a Type,
	pub default: Option<&'a Expr>,
}

#[allow(clippy::needless_collect)] // Inaccurate lint, apparently.
impl ParameterHelperDefintions {
	pub fn new(
		component_generics: &Generics,
		basic_function_generics: &Generics,
		custom_function_generics: &Generics,
		custom_arguments: &[CustomArgument],
		transient_lifetime: &Lifetime,
	) -> Self {
		let mut impl_generics = vec![];
		let argument_types = custom_arguments
			.iter()
			.map(|arg| {
				arg.ty
					.clone()
					.tap_mut(|ty| transform_type(ty, transient_lifetime, &mut impl_generics, true))
			})
			.collect::<Vec<_>>();

		let basic_function_generics_transformed =
			basic_function_generics.clone().tap_mut(|generics| {
				for generic_param in generics.params.iter_mut() {
					transform_generic_param(generic_param, transient_lifetime)
				}
			});

		let basic_function_generics_stripped =
			basic_function_generics.clone().tap_mut(|generics| {
				for generic_param in generics.params.iter_mut() {
					match generic_param {
						GenericParam::Type(type_param) => {
							type_param.bounds = type_param
								.bounds
								.iter()
								.filter(|type_param_bounds| match type_param_bounds {
									TypeParamBound::Trait(_) => (true),
									TypeParamBound::Lifetime(lifetime) => lifetime.ident != "_",
								})
								.cloned()
								.collect()
						}
						GenericParam::Lifetime(lifetime_def) => {
							lifetime_def.bounds = lifetime_def
								.bounds
								.iter()
								.filter(|l| l.ident != "_")
								.cloned()
								.collect()
						}
						GenericParam::Const(_) => (),
					}
				}
			});

		let transient_generics: Generics = parse_quote!(<#transient_lifetime>);

		let custom_function_generics_bounded =
			custom_function_generics.clone().tap_mut(|generics| {
				for generic_param in generics.params.iter_mut() {
					match generic_param {
						GenericParam::Type(type_param) => type_param
							.bounds
							.insert(0, TypeParamBound::Lifetime(transient_lifetime.clone())),
						GenericParam::Lifetime(lifetime_def) => {
							lifetime_def.bounds.insert(0, transient_lifetime.clone())
						}
						GenericParam::Const(_) => (),
					}
				}
			});

		let phantom_args = AngleBracketedGenericArguments {
			colon2_token: None,
			lt_token: <Token![<]>::default(),
			args: iter::once(GenericArgument::Type(Type::Tuple(TypeTuple {
				paren_token: Paren::default(),
				elems: transient_generics
					.params
					.iter()
					.chain(component_generics.params.iter())
					.chain(basic_function_generics.params.iter())
					.filter_map(|param| match param {
						GenericParam::Type(ty_param) => Some(ty_param.ident.to_type()),
						GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
							Some(parse_quote!(&#lifetime()))
						}
						GenericParam::Const(_) => None, // Hopefully fine?
					})
					.collect(),
			})))
			.collect(),
			gt_token: <Token![>]>::default(),
		};

		Self {
			on_parameter_struct: transient_generics
				.add(component_generics)
				.add(&basic_function_generics_transformed)
				.add(custom_function_generics)
				.add(&parse_quote!(<#(#impl_generics),*>))
				.into_owned(),
			parameter_struct_body: FieldsNamed {
				brace_token: Brace::default(),
				named: custom_arguments
					.iter()
					.zip(argument_types.into_iter())
					.map(
						|(
							&CustomArgument {
								attrs,
								ident,
								ty: _,
								default,
							},
							ty,
						)| {
							Field {
								//TODO: Builder docs.
								//TODO?: Better optionals. Something like `ident?: Type` to express `ident: Option<Type> = None` but with the Option stripped for the setter?
								//   Of course the counter-argument here is that I'd like to transition to native Rust named and default parameters eventually,
								//   and it's unlikely that the language will get an option-stripping workaround that doesn't interfere with generic type inference.
								attrs: if let Some(default) = default {
									attrs
										.iter()
										.cloned()
										.chain(iter::once(
											parse_quote!(#[builder(default = #default)]),
										))
										.collect()
								} else {
									attrs.to_vec()
								},
								vis: Visibility::Inherited,
								ident: Some(ident.clone()),
								colon_token: Some(<Token![:]>::default()),
								ty,
							}
						},
					)
					.chain(iter::once(Field {
						attrs: vec![parse_quote!(#[builder(default, setter(skip))])],
						vis: Visibility::Inherited,
						ident: parse_quote!(__asteracea__phantom),
						colon_token: Some(<Token![:]>::default()),
						ty: parse_quote!(::std::marker::PhantomData#phantom_args),
					}))
					.collect(),
			},
			on_function: basic_function_generics_stripped
				.add(custom_function_generics)
				.add(&parse_quote!(<#(#impl_generics),*>))
				.into_owned(),
			for_function_args: AngleBracketedGenericArguments {
				colon2_token: None,
				lt_token: <Token![<]>::default(),
				args: iter::once(GenericArgument::Lifetime(Lifetime {
					apostrophe: Span::mixed_site(),
					ident: Ident::new("_", Span::mixed_site()),
				}))
				.chain(
					component_generics
						.add(basic_function_generics)
						.add(custom_function_generics)
						.params
						.iter()
						.map(|param| param.to_argument()),
				)
				.chain(
					impl_generics
						.iter()
						.map(|type_param| type_param.to_argument()),
				)
				.collect(),
				gt_token: <Token![>]>::default(),
			},
			on_builder_function: transient_generics
				.add(&basic_function_generics_transformed)
				.add(&custom_function_generics_bounded)
				.add(&parse_quote!(<#(#impl_generics),*>))
				.into_owned(),
			for_builder_function_return: AngleBracketedGenericArguments {
				colon2_token: None,
				lt_token: <Token![<]>::default(),
				args: {
					let mut args: Punctuated<GenericArgument, Token![,]> = transient_generics
						.add(component_generics)
						.add(basic_function_generics)
						.add(custom_function_generics)
						.params
						.iter()
						.map(|param| param.to_argument())
						.chain(
							impl_generics
								.iter()
								.map(|type_param| type_param.to_argument()),
						)
						.collect();

					let insert_position = args
						.iter()
						.position(|arg| !matches!(arg, GenericArgument::Lifetime(_)))
						.unwrap_or_else(|| args.len());
					args.insert(
						insert_position,
						GenericArgument::Type(Type::Tuple(TypeTuple {
							paren_token: Paren::default(),
							elems: iter::repeat_with(|| {
								Type::Tuple(TypeTuple {
									paren_token: Paren::default(),
									elems: iter::empty::<Type>().collect(),
								})
							})
							.take(custom_arguments.len())
							.collect::<Punctuated<_, _>>()
							.into_with_trailing(),
						})),
					);
					args
				},
				gt_token: <Token![>]>::default(),
			},
		}
	}
}
