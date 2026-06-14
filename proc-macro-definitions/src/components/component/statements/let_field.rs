use loess::grammar;
use loess_rust::{
	ident::Identifier,
	lex::{
		keywords::{Let, SelfLowercase},
		token::punct::{Dot, Eq, Semi},
	},
};
use loess_rust_opaque::Expression;

grammar! {
	pub struct LetField: PeekFrom, PopFrom {
		r#let: Let,
		self_: SelfLowercase,
		dot: Dot,
		name: Identifier,
		eq: Eq,
		expr: Expression,
		semi: Semi,
	}
}
