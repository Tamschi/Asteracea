use loess::{
	grammar,
	scaffold::{Optimistic, Separated, SquareBrackets},
};
use loess_rust::{
	ident::Identifier,
	lex::{
		keywords::{Async, Dyn},
		token::{
			life::Lifetime,
			punct::{Colon, Comma},
		},
	},
	vis::Visibility,
};
use proc_macro2::TokenTree;

pub type ConstructorArgs = Separated<ConstructorArg, Comma>;
pub type RenderArgs = Separated<RenderArg, Comma>;
pub type SlotArgs = Separated<SlotArg, Comma>;

grammar! {
	pub struct ConstructorArg: PopFrom {
		vis: Option<Visibility>,
		r#dyn: Option<Dyn>,
		pattern: Identifier, //TODO: PatParam,
		colon: Colon,
		r#type: Identifier, //TODO: Type,
	}
	pub struct RenderArg: PopFrom {
		pattern: Identifier, //TODO: PatParam,
		colon: Colon,
		r#type: Identifier, //TODO: Type,
	}
	pub struct SlotArg: PopFrom {
		r#async: Option<Async>,
		label: Lifetime,
		args: Option<SquareBrackets<Optimistic<Separated<RenderArg, Comma>>>>,
	}

	pub struct NotComma: PeekFrom, PopFrom {
		tt: TokenTree,
	}
}
