//! # Why?
//!
//! This is needed to generate function argument container types,
//! in order to use named arguments and argument defaults before they become a language feature.

use syn::{
	AngleBracketedGenericArguments, Binding, Constraint, GenericArgument, Lifetime,
	ParenthesizedGenericArguments, PatType, PathArguments, PathSegment, ReturnType, TraitBound,
	Type, TypeArray, TypeGroup, TypeImplTrait, TypeParamBound, TypeParen, TypePath, TypeReference,
	TypeSlice, TypeTraitObject, TypeTuple,
};

fn apply_to_lifetime(existing_lifetime: &mut Lifetime, lifetime: &Lifetime) -> bool {
	if existing_lifetime.ident == "_" {
		*existing_lifetime = lifetime.clone();
		true
	} else {
		false
	}
}

fn apply_to_path_segments<'a>(
	segments: impl IntoIterator<Item = &'a mut PathSegment>,
	lifetime: &Lifetime,
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
						| GenericArgument::Binding(Binding { ty, .. }) => apply_to_type(ty, lifetime),
						GenericArgument::Constraint(Constraint { bounds, .. }) => {
							apply_to_type_param_bounds(bounds.iter_mut(), lifetime)
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
				inputs
					.iter_mut()
					.fold(false, |acc, input| acc | apply_to_type(input, lifetime))
					| match output {
						ReturnType::Default => false,
						ReturnType::Type(_, ty) => apply_to_type(&mut *ty, lifetime),
					}
			}
		}
	})
}

fn apply_to_type_param_bounds<'a>(
	bounds: impl IntoIterator<Item = &'a mut TypeParamBound>,
	lifetime: &Lifetime,
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
							.fold(false, |acc, l| acc | apply_to_lifetime(l, lifetime))
					})
				}) | apply_to_path_segments(path.segments.iter_mut(), lifetime)
			}
			TypeParamBound::Lifetime(l) => apply_to_lifetime(l, lifetime),
		})
	})
}

fn apply_to_type(ty: &mut Type, lifetime: &Lifetime) -> bool {
	#[allow(clippy::wildcard_in_or_patterns)]
	match ty {
		Type::Array(TypeArray { elem, .. })
		| Type::Paren(TypeParen { elem, .. })
		| Type::Group(TypeGroup { elem, .. })
		| Type::Slice(TypeSlice { elem, .. }) => apply_to_type(elem, lifetime),
		Type::BareFn(_) => todo!("Type::BareFn"),
		Type::ImplTrait(TypeImplTrait { bounds, .. }) => {
			apply_to_type_param_bounds(bounds.iter_mut(), lifetime)
		}
		Type::Infer(_) => todo!("Type::Infer"),
		Type::Never(_) => todo!("Type::Never"),
		Type::Path(TypePath { qself, path }) => {
			qself
				.as_mut()
				.map_or(false, |qself| apply_to_type(&mut *qself.ty, lifetime))
				| apply_to_path_segments(path.segments.iter_mut(), lifetime)
		}
		Type::Ptr(_) => todo!("Type::Ptr"),
		Type::Reference(TypeReference {
			lifetime: l, elem, ..
		}) => {
			(if l.as_ref().map_or(true, |l| l.ident == "_") {
				*l = Some(lifetime.clone());
				true
			} else {
				false
			}) | apply_to_type(&mut *elem, lifetime)
		}
		Type::TraitObject(TypeTraitObject { bounds, .. }) => {
			apply_to_type_param_bounds(bounds.iter_mut(), lifetime)
		}
		Type::Tuple(TypeTuple { elems, .. }) => elems
			.iter_mut()
			.fold(false, |acc, elem| acc | apply_to_type(elem, lifetime)),
		Type::Verbatim(_) => todo!("Type::Verbatim"),
		Type::Macro(_) | _ => {
			// Do nothing and hope for the best.
			false
		}
	}
}

pub fn apply_to_pat_type(mut pat_type: PatType, lifetime: &Lifetime) -> PatType {
	apply_to_type(&mut *pat_type.ty, lifetime); //TODO: Propagate usage flag.
	pat_type
}
