use crate::component_declaration::FieldDefinition;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, spanned::Spanned, ExprPath, Generics, Ident, Result, Visibility};
use unzip_n::unzip_n;

pub struct ParseContext<'a> {
	pub item_visibility: &'a Visibility,
	pub component_name: Option<&'a Ident>,
	pub storage_generics: &'a Generics,
	pub storage_context: StorageContext,
	pub random_items: Vec<TokenStream>,
}

impl<'a> ParseContext<'a> {
	pub fn new_root(
		component_visibility: &'a Visibility,
		component_name: &'a Ident,
		component_generics: &'a Generics,
	) -> Self {
		Self {
			item_visibility: component_visibility,
			component_name: Some(component_name),
			storage_generics: component_generics,
			storage_context: StorageContext {
				type_name: component_name.clone(),
				field_definitions: vec![],
				generated_names: 0,
			},
			random_items: vec![],
		}
	}

	pub fn new_fragment(dummy_visibility: &'a Visibility, dummy_generics: &'a Generics) -> Self {
		Self {
			item_visibility: dummy_visibility,
			component_name: None,
			storage_generics: dummy_generics,
			storage_context: StorageContext {
				type_name: Ident::new("UNUSED", Span::mixed_site()),
				field_definitions: vec![],
				generated_names: 0,
			},
			random_items: vec![],
		}
	}

	pub fn new_nested(&self, type_name: Ident, nested_generics: &'a Generics) -> Self {
		Self {
			item_visibility: self.item_visibility,
			component_name: self.component_name,
			storage_generics: nested_generics,
			storage_context: StorageContext {
				type_name,
				field_definitions: vec![],
				generated_names: 0,
			},
			random_items: vec![],
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

		let where_clause = &generics.where_clause;
		quote_spanned! {type_name.span()=>
			#allow_non_snake_case_on_structure_workaround
			#visibility struct #type_name#generics
			#where_clause
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
