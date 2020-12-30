use crate::component_declaration::FieldDefinition;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{parse::ParseStream, parse2, Expr, ExprPath, Generics, Ident, Result, Visibility};
use unzip_n::unzip_n;

pub struct ParseContext<'a> {
	pub component_name: Option<&'a Ident>,
	pub storage: Expr,
	pub storage_context: StorageContext,
}

impl<'a> ParseContext<'a> {
	pub fn new_root(component_name: &'a Ident) -> Self {
		Self {
			component_name: Some(component_name),
			storage: parse2(quote_spanned!(component_name.span()=> self)).unwrap(),
			storage_context: StorageContext::default(),
		}
	}

	pub fn new_fragment() -> Self {
		Self {
			component_name: None,
			storage: parse2(quote_spanned!(Span::mixed_site()=> self)).unwrap(),
			storage_context: Default::default(),
		}
	}

	pub fn new_nested(&self, storage: Expr) -> Self {
		Self {
			component_name: self.component_name,
			storage,
			storage_context: StorageContext::default(),
		}
	}
}

#[derive(Default)]
pub struct StorageContext {
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
}

pub trait ParseWithContext {
	//WAITING: https://github.com/rust-lang/rust/issues/29661, = Self
	type Output;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output>;
}
