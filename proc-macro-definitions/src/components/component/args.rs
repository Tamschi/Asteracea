use loess::{
	grammar,
	scaffold::{Greedy, Optimistic, Separated, SquareBrackets},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{As, Async, Dyn},
		token::{
			life::Lifetime,
			punct::{Colon, Comma, Eq, Question},
		},
	},
	vis::Visibility,
};
use loess_rust_opaque::{Expression, Pattern, Type};
use proc_macro2::TokenTree;

pub type InitArgs = Separated<InitArg, Comma>;
pub type RenderArgs = Separated<RenderArg, Comma>;
pub type SlotArgs = Separated<SlotArg, Comma>;

grammar! {
	pub struct InitArg: PopFrom {
		pub attrs: Greedy<Vec<OuterAttribute>>,
		pub vis: Option<Visibility>,
		pub r#dyn: Option<Dyn>,
		pub identifier: Identifier,
		pub question: Option<Question>,
		pub expansion: Option<(As, Pattern)>,
		pub colon: Colon,
		pub r#type: Type,
		pub default: Option<(Eq, Expression)>,
	}
	pub struct RenderArg: PopFrom {
		pub attrs: Greedy<Vec<OuterAttribute>>,
		pub identifier: Identifier,
		pub question: Option<Question>,
		pub expansion: Option<(As, Pattern)>,
		pub colon: Colon,
		pub r#type: Type, //TODO: Type,
		pub default: Option<(Eq, Expression)>,
	}
	pub struct SlotArg: PopFrom {
		pub r#async: Option<Async>,
		pub label: Lifetime,
		pub args: Option<SquareBrackets<Optimistic<Separated<RenderArg, Comma>>>>,
	}

	pub struct NotComma: PeekFrom, PopFrom {
		tt: TokenTree,
	}
}
