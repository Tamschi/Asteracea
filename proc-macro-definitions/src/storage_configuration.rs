use std::iter;

use quote::ToTokens;
use syn::{
	parse::{Parse, ParseStream},
	parse2,
	punctuated::{Pair, Punctuated},
	AngleBracketedGenericArguments, Error, ExprPath, GenericArgument, GenericParam, Generics,
	Ident, LifetimeDef, Path, PathArguments, PathSegment, Result, Token, TypeParam, TypePath,
	Visibility, WhereClause,
};
use unquote::unquote;
use wyz::Pipe;

use crate::parse_with_context::StorageContext;

/// ⟦⦃priv‖⦅Visibility⦆⦄ …⦅StorageTypeConfiguration⦆⟧
pub enum StorageConfiguration {
	Anonymous,
	Bound {
		visibility: Visibility,
		field_name: Ident,
		type_configuration: StorageTypeConfiguration,
	},
}

/// ⟦: ⟦struct⟧ … ⟦where …;⟧⟧
#[derive(Clone)]
pub enum StorageTypeConfiguration {
	Anonymous,
	Generated {
		struct_: Token![struct],
		type_name: Ident,
		generics: (Option<Token![::]>, Generics),
	},
	Predefined {
		type_path: ExprPath,
		where_clause: Option<WhereClause>,
	},
}

impl Parse for StorageConfiguration {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self::Bound {
			visibility: if input.parse::<Option<Token![priv]>>().unwrap().is_some() {
				Visibility::Inherited
			} else {
				match input.parse::<Visibility>().unwrap() {
					Visibility::Inherited => return Ok(Self::Anonymous),
					explicit => explicit,
				}
			},
			field_name: input.parse()?,
			type_configuration: input.parse()?,
		})
	}
}

impl Parse for StorageTypeConfiguration {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.parse::<Option<Token![:]>>().unwrap().is_none() {
			Self::Anonymous
		} else if let Some(struct_) = input.parse()? {
			let type_name = input.parse()?;
			let generics = if let Some(colon2) = input.parse::<Option<Token![::]>>().unwrap() {
				let mut generics = input.parse::<Generics>()?;
				if generics.lt_token.is_none() {
					return Err(input.parse::<Token![<]>().unwrap_err());
				}
				generics.where_clause = input.parse()?;
				if generics.where_clause.is_some() {
					unquote!(input, ;);
				}
				(Some(colon2), generics)
			} else {
				(None, Generics::default())
			};
			Self::Generated {
				struct_,
				type_name,
				generics,
			}
		} else {
			let where_clause: Option<WhereClause>;
			unquote! {input,
				#let type_path
				#where_clause
			};
			if where_clause.is_some() {
				unquote!(input, ;);
			}
			Self::Predefined {
				type_path,
				where_clause,
			}
		}
		.pipe(Ok)
	}
}

fn strip_params(
	arguments: &Punctuated<GenericParam, Token![,]>,
) -> Punctuated<GenericParam, Token![,]> {
	arguments
		.pairs()
		.map(|pair| {
			Pair::new(
				match pair.value() {
					GenericParam::Type(t) => GenericParam::Type(TypeParam {
						attrs: vec![],
						ident: t.ident.clone(),
						colon_token: None,
						bounds: Punctuated::default(),
						eq_token: None,
						default: None,
					}),
					GenericParam::Lifetime(l) => GenericParam::Lifetime(LifetimeDef {
						attrs: vec![],
						lifetime: l.lifetime.clone(),
						colon_token: None,
						bounds: Punctuated::default(),
					}),
					GenericParam::Const(c) => GenericParam::Type(TypeParam {
						attrs: vec![],
						ident: c.ident.clone(),
						colon_token: None,
						bounds: Punctuated::default(),
						eq_token: None,
						default: None,
					}),
				},
				pair.punct().cloned().cloned(),
			)
		})
		.collect()
}

fn generic_arguments_to_generic_params(
	arguments: &Punctuated<GenericArgument, Token![,]>,
) -> Result<Punctuated<GenericParam, Token![,]>> {
	arguments
		.pairs()
		.map(|pair| {
			Ok(Pair::new(
				match pair.value() {
					syn::GenericArgument::Lifetime(l) => GenericParam::Lifetime(LifetimeDef {
						attrs: vec![],
						lifetime: l.clone(),
						colon_token: None,
						bounds: Punctuated::default(),
					}),
					syn::GenericArgument::Type(t) => GenericParam::Type(TypeParam {
						attrs: vec![],
						ident: parse2(t.to_token_stream())?,
						colon_token: None,
						bounds: Punctuated::default(),
						eq_token: None,
						default: None,
					}),
					syn::GenericArgument::Binding(_) => {
						todo!("storage configuration generic binding")
					}
					syn::GenericArgument::Constraint(_) => {
						todo!("storage configuration generic constraint")
					}
					syn::GenericArgument::Const(_) => {
						todo!("storage configuration generic const")
					}
				},
				pair.punct().cloned().cloned(),
			))
		})
		.collect()
}

use syn::{ConstParam, Type};

fn generic_arguments(generics: &Generics) -> Result<Punctuated<GenericArgument, Token![,]>> {
	generics
		.params
		.pairs()
		.map(|pair| {
			Pair::new(
				match pair.value() {
					GenericParam::Lifetime(LifetimeDef { attrs, .. })
					| GenericParam::Const(ConstParam { attrs, .. })
					| GenericParam::Type(TypeParam { attrs, .. })
						if !attrs.is_empty() =>
					{
						return Err(Error::new_spanned(
							attrs.first().unwrap(),
							"Attributes are not supported here.",
						))
					}
					GenericParam::Const(ConstParam { ident, .. })
					| GenericParam::Type(TypeParam { ident, .. }) => GenericArgument::Type(Type::Path(TypePath {
						qself: None,
						path: ident.clone().into(),
					})),
					GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
						GenericArgument::Lifetime(lifetime.clone())
					}
				},
				pair.punct().copied().copied(),
			)
			.pipe(Ok)
		})
		.collect::<Result<Punctuated<_, _>>>()?
		.pipe(Ok)
}

impl StorageConfiguration {
	pub fn visibility(&self) -> Visibility {
		match self {
			StorageConfiguration::Anonymous => Visibility::Inherited,
			StorageConfiguration::Bound { visibility, .. } => visibility.clone(),
		}
	}

	pub fn field_name(&self) -> Option<&Ident> {
		match self {
			StorageConfiguration::Anonymous => None,
			StorageConfiguration::Bound { field_name, .. } => Some(field_name),
		}
	}

	pub fn type_configuration(&self) -> StorageTypeConfiguration {
		match self {
			StorageConfiguration::Anonymous => StorageTypeConfiguration::Anonymous,
			StorageConfiguration::Bound {
				type_configuration, ..
			} => type_configuration.clone(),
		}
	}
}

impl StorageTypeConfiguration {
	pub fn type_path(&self, container: &StorageContext, field_name: &Ident) -> Result<ExprPath> {
		match self {
			StorageTypeConfiguration::Anonymous => ExprPath {
				attrs: vec![],
				qself: None,
				path: container.generated_type_name(field_name).into(),
			},
			StorageTypeConfiguration::Generated {
				type_name,
				generics,
				..
			} => ExprPath {
				attrs: vec![],
				qself: None,
				path: Path {
					leading_colon: None,
					segments: iter::once(PathSegment {
						ident: type_name.clone(),
						arguments: if generics.0.is_none() {
							PathArguments::None
						} else {
							PathArguments::AngleBracketed(AngleBracketedGenericArguments {
								colon2_token: generics.0,
								lt_token: generics.1.lt_token.unwrap(),
								args: generic_arguments(&generics.1)?,
								gt_token: generics.1.gt_token.unwrap(),
							})
						},
					})
					.collect(),
				},
			},
			StorageTypeConfiguration::Predefined { type_path, .. } => type_path.clone(),
		}
		.pipe(Ok)
	}

	pub fn generics(&self) -> Result<Option<Generics>> {
		match self {
			StorageTypeConfiguration::Anonymous => None,
			StorageTypeConfiguration::Generated {
				generics: (_, generics),
				..
			} => Some(generics.clone()),
			StorageTypeConfiguration::Predefined {
				type_path,
				where_clause,
			} => {
				let arguments = &type_path.path.segments.last().unwrap().arguments;
				let mut generics = match arguments {
					PathArguments::None => Generics::default(),
					PathArguments::AngleBracketed(a_bra_args) => Generics {
						lt_token: Some(a_bra_args.lt_token),
						params: generic_arguments_to_generic_params(&a_bra_args.args)?,
						gt_token: Some(a_bra_args.gt_token),
						where_clause: None,
					},
					PathArguments::Parenthesized(p) => {
						return Err(Error::new_spanned(
							p,
							"Parenthesized generic arguments are not supported in this position.",
						));
					}
				};
				generics.where_clause = where_clause.as_ref().cloned();
				Some(generics)
			}
		}
		.pipe(Ok)
	}
}
