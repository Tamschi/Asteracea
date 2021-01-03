use syn::{
	parse::{Parse, ParseStream},
	ExprPath, Generics, Ident, Result, Token, Visibility, WhereClause,
};
use unquote::unquote;
use wyz::Pipe;

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
enum StorageTypeConfiguration {
	Anonymous,
	Generated {
		struct_: Token![struct],
		type_name: Ident,
		generics: Generics,
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
			let generics = if input.parse::<Option<Token![::]>>().unwrap().is_some() {
				let mut generics = input.parse::<Generics>()?;
				generics.where_clause = input.parse()?;
				if generics.where_clause.is_some() {
					unquote!(input, ;);
				}
				generics
			} else {
				Generics::default()
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
