use loess::{
	grammar,
	scaffold::{Greedy, Optimistic, Separated, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{Async, Dyn},
		token::{
			life::Lifetime,
			punct::{Colon, Comma, Question},
		},
	},
	vis::Visibility,
};
use loess_rust_opaque::Type;
use proc_macro2::TokenTree;

pub type ConstructorArgs = Separated<ConstructorArg, Comma>;
pub type RenderArgs = Separated<RenderArg, Comma>;
pub type SlotArgs = Separated<SlotArg, Comma>;

grammar! {
	pub struct ConstructorArg: PopFrom {
		attrs: Greedy<Vec<OuterAttribute>>,
		vis: Option<Visibility>,
		r#dyn: Option<Dyn>,
		pattern: Identifier, //TODO: PatParam,
		colon: Colon,
		r#type: Type,
	}
	pub struct RenderArg: PopFrom {
		pattern: Identifier, //TODO: PatParam,
		question: Option<Question>,
		colon: Colon,
		r#type: Type, //TODO: Type,
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
