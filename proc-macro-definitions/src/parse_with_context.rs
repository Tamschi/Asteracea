use crate::component_declaration::FieldDefinition;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{
	parse::ParseStream, spanned::Spanned, Attribute, Error, ExprPath, Generics, Ident, Result,
	Visibility,
};
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
		attributes: &[Attribute],
		visibility: &Visibility,
		type_name: &Ident,
		generics: &Generics,
	) -> Result<TokenStream> {
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

		let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

		let explicit_default_drop = if self.field_definitions.iter().any(|f| f.structurally_pinned)
		{
			for attribute in attributes {
				if let Some(path) = attribute.path.get_ident() {
					if path == "repr" {
						return Err(Error::new_spanned(path, "Custom `repr` not allowed due to field pin projection. (Box any child components to get around this.)"));
					}
				}
			}

			Some(quote_spanned! {type_name.span()=>
				/// Asteracea implements some level of structural pinning for any child components not pinned by a `box <…>` expression.
				/// As such, no custom Drop implementation is possible directly on this storage context.
				///
				/// See [`std::pin` - Pinning *is* structural for `field`](https://doc.rust-lang.org/stable/std/pin/index.html#pinning-is-structural-for-field) for more information.
				///
				/// > **Note:**
				/// >
				/// > Components may still be [`Unpin`](`::std::marker::Unpin`) - unless any direct child components are `!Unpin`.
				/// > The generated structural pinning implementation contains a static assertion in this regard,
				/// > which should only fail if you manually implement [`Unpin`](`::std::marker::Unpin`) on the respective storage context.
				impl#impl_generics ::core::ops::Drop for #type_name#type_generics
					#where_clause {
						fn drop(&mut self) {
							let _ = self;
						}
				}
			})
		} else {
			None
		};

		let structural_pinning = self
			.field_definitions
			.iter()
			.filter_map(|field| {
				if field.structurally_pinned {
					let asteracea =
						crate::asteracea_ident(field.name.span().resolved_at(Span::mixed_site()));
					let FieldDefinition {
						attributes,
						visibility,
						name,
						field_type,
						initial_value: _,
						structurally_pinned: _,
					} = field;
					let pinned_name = Ident::new(&format!("{}_pinned", name), name.span());
					Some(quote_spanned! {field.name.span()=>
						#(#attributes)*
						#visibility fn #pinned_name(self: ::std::pin::Pin<&Self>) -> ::std::pin::Pin<&#field_type> {
							//TODO!: Reactivate this somehow! The library is unsound without it.
							// {
							// 	type StorageContext = Self;
							// 	type Field = #field_type;
							// 	const legal: bool = !::#asteracea::impls::impls!(Field: !::std::marker::Unpin) || !::#asteracea::impls::impls!(StorageContext: ::std::marker::Unpin);
							// 	::#asteracea::static_assertions::const_assert!(legal);
							// };

							unsafe {
								self.map_unchecked(|this| &this.#name)
							}
						}
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		let structural_pinning = if structural_pinning.is_empty() {
			None
		} else {
			Some(quote_spanned! {type_name.span()=>
				impl#impl_generics #type_name#type_generics #where_clause {
					#(#structural_pinning)*
				}
			})
		};

		Ok(quote_spanned! {type_name.span()=>
			#(#attributes)*
			#allow_non_snake_case_on_structure_workaround
			#visibility struct #type_name#generics
			#where_clause
			{
				#(
					#(#field_attributes)*
					#field_visibilities #field_names: #field_types,
				)*
			}

			#explicit_default_drop

			#structural_pinning
		})
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
