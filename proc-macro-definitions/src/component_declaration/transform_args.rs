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
						| GenericArgument::Binding(Binding { ty, .. }) => transform_type(ty, lifetime),
						GenericArgument::Constraint(Constraint { bounds, .. }) => {
							transform_type_param_bounds(bounds.iter_mut(), lifetime)
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
					.fold(false, |acc, input| acc | transform_type(input, lifetime))
					| match output {
						ReturnType::Default => false,
						ReturnType::Type(_, ty) => transform_type(&mut *ty, lifetime),
					}
			}
		}
	})
}

fn transform_type_param_bounds<'a>(
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
							.fold(false, |acc, l| acc | transform_lifetime(l, lifetime))
					})
				}) | transform_path_segments(path.segments.iter_mut(), lifetime)
			}
			TypeParamBound::Lifetime(l) => transform_lifetime(l, lifetime),
		})
	})
}

fn transform_type(ty: &mut Type, lifetime: &Lifetime) -> bool {
	#[allow(clippy::wildcard_in_or_patterns)]
	match ty {
		Type::Array(TypeArray { elem, .. })
		| Type::Paren(TypeParen { elem, .. })
		| Type::Group(TypeGroup { elem, .. })
		| Type::Slice(TypeSlice { elem, .. }) => transform_type(elem, lifetime),
		Type::BareFn(_) => todo!("Type::BareFn"),
		Type::ImplTrait(TypeImplTrait { bounds, .. }) => {
			transform_type_param_bounds(bounds.iter_mut(), lifetime)
		}
		Type::Infer(_) => todo!("Type::Infer"),
		Type::Never(_) => todo!("Type::Never"),
		Type::Path(TypePath { qself, path }) => {
			qself
				.as_mut()
				.map_or(false, |qself| transform_type(&mut *qself.ty, lifetime))
				| transform_path_segments(path.segments.iter_mut(), lifetime)
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
			}) | transform_type(&mut *elem, lifetime)
		}
		Type::TraitObject(TypeTraitObject { bounds, .. }) => {
			transform_type_param_bounds(bounds.iter_mut(), lifetime)
		}
		Type::Tuple(TypeTuple { elems, .. }) => elems
			.iter_mut()
			.fold(false, |acc, elem| acc | transform_type(elem, lifetime)),
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

pub fn transform_pat_type(mut pat_type: PatType, lifetime: &Lifetime) -> PatType {
	transform_type(&mut *pat_type.ty, lifetime); //TODO: Propagate usage flag.
	pat_type
}
