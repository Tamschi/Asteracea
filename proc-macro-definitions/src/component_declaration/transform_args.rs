//! # Why?
//!
//! This is needed to generate function argument container types,
//! in order to use named arguments and argument defaults before they become a language feature.

use std::{iter, mem};
use syn::{
	spanned::Spanned as _, AngleBracketedGenericArguments, Binding, Constraint, GenericArgument,
	Ident, Lifetime, ParenthesizedGenericArguments, PatType, Path, PathArguments, PathSegment,
	ReturnType, Token, TraitBound, Type, TypeArray, TypeGroup, TypeParam, TypeParamBound,
	TypeParen, TypePath, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
};

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

pub fn transform_pat_type(
	mut pat_type: PatType,
	lifetime: &Lifetime,
	impl_generics: &mut Vec<TypeParam>,
	adjust_lifetimes: bool,
) -> PatType {
	transform_type(&mut *pat_type.ty, lifetime, impl_generics, adjust_lifetimes); //TODO: Propagate usage flag.
	pat_type
}
