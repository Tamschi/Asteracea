use crate::{
	component_declaration::FieldDefinition, storage_configuration::StorageTypeConfiguration,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
	parse::ParseStream,
	punctuated::{Pair, Punctuated},
	spanned::Spanned,
	ExprPath, Field, GenericParam, Generics, Ident, Item, LifetimeDef, Result, Token, Type,
	TypeParam, Visibility,
};

pub struct ParseContext<'a> {
	pub item_visibility: &'a Visibility,
	pub component_name: Option<&'a Ident>,
	pub storage_generics: &'a Generics,
	pub storage_context: StorageContext,
	pub assorted_items: Vec<Item>,
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
			assorted_items: vec![],
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
			assorted_items: vec![],
		}
	}

	pub fn new_nested(
		&self,
		type_name_as_if_generated: Ident,
		nested_generics: &'a Generics,
	) -> Self {
		Self {
			item_visibility: self.item_visibility,
			component_name: self.component_name,
			storage_generics: nested_generics,
			storage_context: StorageContext {
				type_name: type_name_as_if_generated,
				field_definitions: vec![],
				generated_names: 0,
			},
			assorted_items: vec![],
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

	pub fn value(&self, generated_type: bool, type_path: &ExprPath) -> TokenStream {
		let (field_names, field_values) = self
			.field_definitions()
			.map(|c| (&c.name, &c.initial_value))
			.unzip::<_, _, Vec<_>, Vec<_>>();

		// Workaround until min_specialization lands. See above.
		let phantom_pinned = if generated_type
			&& !type_path
				.to_token_stream()
				.to_string()
				.contains("__Asteracea__")
			&& type_path
				.path
				.segments
				.last()
				.expect("StorageContext.value(…): Called with an empty `type_path`, somehow.")
				.arguments
				.is_empty() && self
			.field_definitions
			.iter()
			.any(|f| f.structurally_pinned)
		{
			Some(
				quote_spanned! {type_path.span().resolved_at(Span::mixed_site())=> __Asteracea__pinned: ::std::marker::PhantomPinned,},
			)
		} else {
			None
		};

		quote_spanned! {type_path.span().resolved_at(Span::mixed_site())=>
			#type_path {
				#(#field_names: (#field_values),)* // The parentheses around #field_values stop the grammar from breaking as much if no value is provided.
				#phantom_pinned
			}
		}
	}

	pub fn fields(
		&self,
		configuration: &StorageTypeConfiguration,
		container_generics: &Generics,
	) -> Vec<Field> {
		let mut fields: Vec<Field> = self
			.field_definitions
			.iter()
			.map(|f| Field {
				attrs: f.attributes.clone(),
				vis: f.visibility.clone(),
				ident: Some(f.name.clone()),
				colon_token: Some(Token![:](f.name.span())),
				ty: Type::Verbatim(f.field_type.clone()),
			})
			.collect();
		if configuration.use_implicit_generics() {
			let span = self.type_name.span().resolved_at(Span::mixed_site());
			let phantom_params = strip_params(&container_generics.params);

			fields.push(Field {
				attrs: vec![],
				vis: Visibility::Inherited,
				ident: Some(Ident::new("__Asteracea__phantom", span)),
				colon_token: Some(Token![:](span)),
				ty: Type::Verbatim(
					quote_spanned!(span=> ::std::marker::PhantomData<(#phantom_params)>),
				),
			})
		}
		fields
	}
}

pub trait ParseWithContext {
	//WAITING: https://github.com/rust-lang/rust/issues/29661, = Self
	type Output;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output>;
}

fn strip_params(
	params: &Punctuated<GenericParam, Token![,]>,
) -> Punctuated<GenericParam, Token![,]> {
	params
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