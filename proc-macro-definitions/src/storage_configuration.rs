use syn::{
	parse::{Parse, ParseStream},
	ExprPath, Generics, Ident, Result, Token, Visibility, WhereClause,
};
use unquote::unquote;
use wyz::Pipe;

pub struct StorageConfiguration {
	visibility: Visibility,
	field_name: Ident,
	accessible_type: Option<(Token![:], StorageTypeConfiguration)>,
}

enum StorageTypeConfiguration {
	Generated {
		struct_: Token![struct],
		type_name: Ident,
		generics_double_colon: Option<Token![::]>,
		generics: Generics,
		where_clause_semicolon: Option<Token![;]>,
	},
	Predefined {
		type_path: ExprPath,
		where_clause: Option<(WhereClause, Token![;])>,
	},
}

impl StorageConfiguration {
	fn parse(input: ParseStream) -> Result<Option<Self>> {
		Ok(Some(Self {
			visibility: if input.parse::<Option<Token![priv]>>().unwrap().is_some() {
				Visibility::Inherited
			} else {
				match input.parse::<Visibility>().unwrap() {
					Visibility::Inherited => return Ok(None),
					explicit => explicit,
				}
			},
			field_name: input.parse()?,
			accessible_type: if let Some(colon) = input.parse()? {
				Some((colon, input.parse()?))
			} else {
				None
			},
		}))
	}
}

impl Parse for StorageTypeConfiguration {
	fn parse(input: ParseStream) -> Result<Self> {
		unquote! {input,
			#let struct_
			#let type_name
		};
		if let Some(struct_) = struct_ {
			let generics_double_colon: Option<Token![::]> = input.parse()?;
			let generics = if generics_double_colon.is_some() {
				let mut generics = input.parse::<Generics>()?;
				generics.where_clause = input.parse()?;
				generics
			} else {
				Generics::default()
			};
			let where_clause_semicolon = if generics.where_clause.is_some() {
				Some(input.parse()?)
			} else {
				None
			};
			Self::Generated {
				struct_,
				type_name,
				generics_double_colon,
				generics,
				where_clause_semicolon,
			}
		} else {
			Self::Predefined {
				type_path: input.parse()?,
				where_clause: input
					.parse::<Option<WhereClause>>()?
					.map(|where_clause| Result::Ok((where_clause, input.parse()?)))
					.transpose()?,
			}
		}
		.pipe(Ok)
	}
}
