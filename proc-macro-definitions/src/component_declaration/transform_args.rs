//! # Why?
//!
//! This is needed to generate function argument container types,
//! in order to use named arguments and argument defaults before they become a language feature.

use std::{iter, mem};
use syn::{spanned::Spanned as _, Ident};
use syn::{
	AngleBracketedGenericArguments, Binding, Constraint, GenericArgument, Lifetime,
	ParenthesizedGenericArguments, PatType, Path, PathArguments, PathSegment, ReturnType, Token,
	TraitBound, Type, TypeArray, TypeGroup, TypeParam, TypeParamBound, TypeParen, TypePath,
	TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
};

fn transform_lifetime(existing_lifetime: &mut Lifetime, lifetime: &Lifetime) -> bool {
	if existing_lifetime.ident == "_" {
		*existing_lifetime = lifetime.clone();
		true
	} else {
		false
	}
}

fn transform_path_segments<'a>(
	segments: impl IntoIterator<Item = &'a mut PathSegment>,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
) -> bool {
	segments.into_iter().fold(false, |acc, segment| {
		acc | match &mut segment.arguments {
			PathArguments::None => false,
			PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
				args.iter_mut().fold(false, |acc, arg| {
					acc | match arg {
						GenericArgument::Lifetime(l) => {
							if l.ident == "_" {
								*l = lifetime.clone();
								true
							} else {
								false
							}
						}
						GenericArgument::Type(ty)
						| GenericArgument::Binding(Binding { ty, .. }) => transform_type(ty, lifetime, impl_generics),
						GenericArgument::Constraint(Constraint { bounds, .. }) => {
							transform_type_param_bounds(bounds.iter_mut(), lifetime, impl_generics)
						}
						GenericArgument::Const(_) => {
							// Do nothing and hope for the best.
							false
						}
					}
				})
			}
			PathArguments::Parenthesized(ParenthesizedGenericArguments {
				inputs, output, ..
			}) => {
				inputs.iter_mut().fold(false, |acc, input| {
					acc | transform_type(input, lifetime, impl_generics)
				}) | match output {
					ReturnType::Default => false,
					ReturnType::Type(_, ty) => transform_type(&mut *ty, lifetime, impl_generics),
				}
			}
		}
	})
}

fn transform_type_param_bounds<'a>(
	bounds: impl IntoIterator<Item = &'a mut TypeParamBound>,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
) -> bool {
	bounds.into_iter().fold(false, |acc, b| {
		acc | (match b {
			TypeParamBound::Trait(TraitBound {
				lifetimes, path, ..
			}) => {
				lifetimes.as_mut().map_or(false, |l| {
					l.lifetimes.iter_mut().fold(false, |acc, l| {
						acc | l
							.bounds
							.iter_mut()
							.fold(false, |acc, l| acc | transform_lifetime(l, lifetime))
					})
				}) | transform_path_segments(path.segments.iter_mut(), lifetime, impl_generics)
			}
			TypeParamBound::Lifetime(l) => transform_lifetime(l, lifetime),
		})
	})
}

fn transform_type(ty: &mut Type, lifetime: &Lifetime, impl_generics: &mut Vec<TypeParam>) -> bool {
	#[allow(clippy::wildcard_in_or_patterns)]
	match ty {
		Type::Array(TypeArray { elem, .. })
		| Type::Paren(TypeParen { elem, .. })
		| Type::Group(TypeGroup { elem, .. })
		| Type::Slice(TypeSlice { elem, .. }) => transform_type(elem, lifetime, impl_generics),
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

			let applied_lifetime =
				transform_type_param_bounds(bounds.iter_mut(), lifetime, impl_generics);

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
			qself.as_mut().map_or(false, |qself| {
				transform_type(&mut *qself.ty, lifetime, impl_generics)
			}) | transform_path_segments(path.segments.iter_mut(), lifetime, impl_generics)
		}
		Type::Ptr(_) => {
			// Not actually allowed, but will error correctly, I think.
			false
		}
		Type::Reference(TypeReference {
			lifetime: l, elem, ..
		}) => {
			(if l.as_ref().map_or(true, |l| l.ident == "_") {
				*l = Some(lifetime.clone());
				true
			} else {
				false
			}) | transform_type(&mut *elem, lifetime, impl_generics)
		}
		Type::TraitObject(TypeTraitObject { bounds, .. }) => {
			transform_type_param_bounds(bounds.iter_mut(), lifetime, impl_generics)
		}
		Type::Tuple(TypeTuple { elems, .. }) => elems.iter_mut().fold(false, |acc, elem| {
			acc | transform_type(elem, lifetime, impl_generics)
		}),
		Type::Verbatim(_) | Type::Macro(_) => {
			// Do nothing and hope for the best.
			false
		}
		_ => {
			//TODO: Warn about unhandled type gammar if possible.
			false
		}
	}
}

pub fn transform_pat_type(
	mut pat_type: PatType,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
) -> PatType {
	transform_type(&mut *pat_type.ty, lifetime, impl_generics); //TODO: Propagate usage flag.
	pat_type
}
