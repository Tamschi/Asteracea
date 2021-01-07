use std::iter;

use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::{Parse, ParseStream},
	parse2,
	punctuated::{Pair, Punctuated},
	token::{Brace, Bracket},
	AngleBracketedGenericArguments, AttrStyle, Attribute, Error, ExprPath, Fields, FieldsNamed,
	GenericArgument, GenericParam, Generics, Ident, ImplItem, ImplItemMethod, Item, ItemImpl,
	ItemStruct, LifetimeDef, Path, PathArguments, PathSegment, Result, Token, TypeParam, TypePath,
	Visibility, WhereClause,
};
use unquote::unquote;
use wyz::Pipe;

use crate::{component_declaration::FieldDefinition, parse_with_context::StorageContext};

/// ⟦⦃priv‖⦅Visibility⦆⦄ …⦅StorageTypeConfiguration⦆⟧
#[allow(clippy::large_enum_variant)]
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

impl StorageTypeConfiguration {
	pub fn new_component_root(ident: Ident, generics: Generics) -> Self {
		let span = ident.span();
		Self::Generated {
			struct_: Token![struct](span),
			type_name: ident,
			generics: (
				// Never inherits anything, so the generics are always as if explicit.
				Some(Token![::](span)),
				Generics {
					lt_token: generics.lt_token.or_else(|| Some(Token![<](span))),
					params: generics.params,
					gt_token: generics.gt_token.or_else(|| Some(Token![>](span))),
					where_clause: generics.where_clause,
				},
			),
		}
	}
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
	pub fn type_path(
		&self,
		container: &StorageContext,
		field_name: &Ident,
		parent_generics: &Generics,
	) -> Result<ExprPath> {
		let span = field_name.span();
		match self {
			StorageTypeConfiguration::Anonymous => ExprPath {
				attrs: vec![],
				qself: None,
				path: Path {
					leading_colon: None,
					segments: iter::once(PathSegment {
						ident: container.generated_type_name(field_name),
						arguments: if parent_generics.params.is_empty() {
							PathArguments::None
						} else {
							PathArguments::AngleBracketed(AngleBracketedGenericArguments {
								colon2_token: Some(Token![::](span)),
								lt_token: parent_generics
									.lt_token
									.as_ref()
									.cloned()
									.unwrap_or_else(|| Token![<](span)),
								args: generic_arguments(&parent_generics)?,
								gt_token: parent_generics
									.gt_token
									.as_ref()
									.cloned()
									.unwrap_or_else(|| Token![>](span)),
							})
						},
					})
					.collect(),
				},
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

	pub fn type_is_generated(&self) -> bool {
		match self {
			StorageTypeConfiguration::Anonymous => true,
			StorageTypeConfiguration::Generated { .. } => true,
			StorageTypeConfiguration::Predefined { .. } => false,
		}
	}

	pub fn use_implicit_generics(&self) -> bool {
		match self {
			StorageTypeConfiguration::Anonymous => true,
			StorageTypeConfiguration::Generated { generics, .. } => generics.0.is_none(),
			StorageTypeConfiguration::Predefined { .. } => false,
		}
	}

	pub fn struct_(&self) -> Option<&Token![struct]> {
		match self {
			StorageTypeConfiguration::Anonymous => None,
			StorageTypeConfiguration::Generated { struct_, .. } => Some(&struct_),
			StorageTypeConfiguration::Predefined { .. } => None,
		}
	}

	pub fn struct_definition(
		&self,
		mut attributes: Vec<Attribute>,
		visibility: Visibility,
		ident: Ident,
		contents: &StorageContext,
		parent_generics: &Generics,
	) -> Result<Vec<Item>> {
		let span = ident.span();
		let generics = self.generics()?.unwrap_or_else(|| parent_generics.clone());

		let fields = contents.fields(self, &generics);

		//TODO: Unsound! Restore safety asserts. (I.e.: Assert that the surrounding type isn't `Unpin`!)

		let structural_pinning_fns = contents
			.field_definitions()
			.map(|f| {
				let f_visibility = &f.visibility;
				let f_name = &f.name;
				let f_type = &f.field_type;
				let fn_name = Ident::new(&format!("{}_pinned", &f_name), span);
				parse2::<ImplItemMethod>(quote_spanned! {span=>
					#f_visibility fn #fn_name(self: ::std::pin::Pin<&Self>) -> ::std::pin::Pin<&#f_type> {
						unsafe { self.map_unchecked(|this| &this.#f_name) }
					}
				})
				.expect("structural pinning method")
			})
			.map(ImplItem::Method)
			.collect();

		let fields = Fields::Named(FieldsNamed {
			brace_token: Brace(span),
			named: fields
				.into_iter()
				.map(|f| Pair::Punctuated(f, Token![,](span)))
				.collect(),
		});

		#[allow(clippy::blocks_in_if_conditions)]
		if fields.iter().any(|f| {
			f.ident
				.as_ref()
				.expect("struct definition field ident")
				.to_string()
				.contains("__Asteracea__")
		}) {
			attributes.push(allow_non_snake_case())
		}

		vec![
			Item::Struct(ItemStruct {
				attrs: attributes,
				vis: visibility,
				struct_token: self
					.struct_()
					.cloned()
					.unwrap_or_else(|| Token![struct](span)),
				ident: ident.clone(),
				fields,
				generics: generics.clone(),
				semi_token: None,
			}),
			Item::Impl(ItemImpl {
				attrs: vec![],
				defaultness: None,
				unsafety: None,
				impl_token: Token![impl](span),
				generics,
				trait_: None,
				self_ty: Box::new(Type::Path(TypePath {
					qself: None,
					path: ident.into(),
				})),
				brace_token: Brace(span),
				items: structural_pinning_fns,
			}),
		]
		.pipe(Ok)
	}
}

fn allow_non_snake_case() -> Attribute {
	let span = Span::mixed_site();
	Attribute {
		pound_token: Token![#](span),
		style: AttrStyle::Outer,
		bracket_token: Bracket(span),
		path: Ident::new("allow", span).into(),
		tokens: quote_spanned! (span=> (non_snake_case)),
	}
}
