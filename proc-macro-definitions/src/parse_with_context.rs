use crate::component_declaration::{FieldDefinition, TypeLevelFieldDefinition};
use syn::{parse::ParseStream, Ident, Result};

#[derive(Default)]
pub struct ParseContext {
	pub component_name: Option<Ident>,
	pub static_shared: Vec<TypeLevelFieldDefinition>,
	pub allow_non_snake_case_on_structure_workaround: bool, // Workaround since the attribute isn't respected on specific fields.
	pub field_definitions: Vec<FieldDefinition>,
	pub event_binding_count: usize,
}

pub trait ParseWithContext {
	//WAITING: https://github.com/rust-lang/rust/issues/29661, = Self
	type Output;
	fn parse_with_context(input: ParseStream<'_>, cx: &mut ParseContext) -> Result<Self::Output>;
}
