use std::borrow::Cow;

use crate::component_declaration::FieldDefinition;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{
	parse::ParseStream, parse2, spanned::Spanned, Expr, ExprPath, Generics, Ident, Result,
	Visibility,
};
use unzip_n::unzip_n;

pub struct ParseContext<'a> {
	pub component_name: Option<&'a Ident>,
	pub storage: Cow<'a, Expr>,
	pub storage_context: StorageContext,
}

impl<'a> ParseContext<'a> {
	pub fn new_root(component_name: &'a Ident) -> Self {
		Self {
			component_name: Some(component_name),
			storage: Cow::Owned(parse2(quote_spanned!(component_name.span()=> self)).unwrap()),
			storage_context: StorageContext {
				type_name: component_name.clone(),
				field_definitions: vec![],
				generated_names: 0,
			},
		}
	}

	pub fn new_fragment() -> Self {
		Self {
			component_name: None,
			storage: Cow::Owned(parse2(quote_spanned!(Span::mixed_site()=> self)).unwrap()),
			storage_context: StorageContext {
				type_name: Ident::new("UNUSED", Span::mixed_site()),
				field_definitions: vec![],
				generated_names: 0,
			},
		}
	}

	pub fn new_nested(&self, storage: &'a Expr, type_name: Ident) -> Self {
		Self {
			component_name: self.component_name,
			storage: Cow::Borrowed(storage),
			storage_context: StorageContext {
				type_name,
				field_definitions: vec![],
				generated_names: 0,
			},
		}
	}
}

pub struct StorageContext {
	type_name: Ident,
	field_definitions: Vec<FieldDefinition>,
	generated_names: usize,
}

impl StorageContext {
	/// Generates a new field name unique in the current [`StorageContext`] located at `span`.
	///
	/// The returned [`Ident`] instance always uses [`mixed_site`](`Span::mixed_site`) resolution.
	pub fn next_field(&mut self, span: Span) -> Ident {
		let field = Ident::new(
			&format!("__Asteracea__{}", self.generated_names),
			span.resolved_at(Span::mixed_site()),
		);
		self.generated_names += 1;
		field
	}

	pub fn generated_type_name(&self, field_name: &Ident) -> Ident {
		Ident::new(
			&format!("{}__Asteracea__Field_{}", self.type_name, field_name),
			field_name.span(),
		)
	}

	pub fn push(&mut self, field_definition: FieldDefinition) {
		self.field_definitions.push(field_definition)
	}

	pub fn field_definitions(&self) -> impl Iterator<Item = &FieldDefinition> {
		self.field_definitions.iter()
	}

	pub fn type_definition(
		&self,
		visibility: &Visibility,
		type_name: &Ident,
		generics: &Generics,
	) -> TokenStream {
		let allow_non_snake_case_on_structure_workaround = if self.generated_names > 0 {
			Some(quote_spanned! (type_name.span()=> #[allow(non_snake_case)]))
		} else {
			None
		};

		unzip_n!(4);

		let (field_attributes, field_visibilities, field_names, field_types) = self
			.field_definitions
			.iter()
			.map(|f| (&f.attributes, &f.visibility, &f.name, &f.field_type))
			.unzip_n_vec();

		quote_spanned! {type_name.span()=>
			#allow_non_snake_case_on_structure_workaround
			#visibility struct #type_name#generics
			{
				#(
					#(#field_attributes)*
					#field_visibilities #field_names: #field_types,
				)*
			}
		}
	}

	pub fn value(&self, type_path: &ExprPath) -> TokenStream {
		let (field_names, field_values) = self
			.field_definitions()
			.map(|c| (&c.name, &c.initial_value))
			.unzip::<_, _, Vec<_>, Vec<_>>();

		quote_spanned! {type_path.span().resolved_at(Span::mixed_site())=>
			#type_path {
				#(#field_names: (#field_values),)* // The parentheses around #field_values stop the grammar from breaking as much if no value is provided.
			}
		}
	}
}

pub trait ParseWithContext {
	//WAITING: https://github.com/rust-lang/rust/issues/29661, = Self
	type Output;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output>;
}
