use loess::{
	SimpleSpanned, grammar,
	scaffold::{Greedy, Optimistic, Parentheses, Separated, SquareBrackets, UpTo},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{As, Async, Dyn, Mut},
		token::{
			life::Lifetime,
			punct::{Colon, Comma, Eq, Question},
		},
	},
	vis::Visibility,
};
use loess_rust_opaque::{Expression, Pattern, Type};
use proc_macro2::{Span, TokenStream, TokenTree};

pub type RenderArgs = Separated<RenderArg, Comma>;
pub type SlotArgs = Separated<SlotArg, Comma>;

grammar! {
	pub struct InitArgs: PopFrom {
		//TODO: Better lookahead to allow attributes and visibility.
		pub args: UpTo<DynOrParenDyn, Separated<InitArg, Comma>>,
		pub deps: Separated<InitDep, Comma>,
	}

	pub struct InitArg: PopFrom {
		pub attrs: Greedy<Vec<OuterAttribute>>,
		pub vis: Option<Visibility>,
		pub r#mut: Option<Mut>,
		pub identifier: Identifier,
		pub question: Option<Question>,
		pub expansion: Option<(As, Pattern)>,
		pub r#type: Option<(Colon, Type)>,
		pub default: Option<(Eq, Expression)>,
	}

	pub struct InitDep: PopFrom {
		pub attrs: Greedy<Vec<OuterAttribute>>,
		pub vis: Option<Visibility>,
		pub r#dyn: DynOrParenDyn,
		pub r#mut: Option<Mut>, //TODO: Can this be part of `pattern`?
		pub pattern: Pattern,
		pub question: Option<Question>,
		pub colon: Colon,
		pub r#type: Type,
		pub default: Option<(Eq, Expression)>,
	}

	pub enum DynOrParenDyn: PeekFrom, PopFrom {
		Dyn(Dyn),
		ParenDyn(Parentheses<Dyn>),
	} else "Expected `dyn` or `(dyn)`.";

	pub struct RenderArg: PopFrom {
		pub attrs: Greedy<Vec<OuterAttribute>>,
		pub r#mut: Option<Mut>,
		pub identifier: Identifier,
		pub question: Option<Question>,
		pub expansion: Option<(As, Pattern)>,
		pub r#type: Option<(Colon, Type)>,
		pub default: Option<(Eq, Expression)>,
	}
	pub struct SlotArg: PopFrom {
		pub r#async: Option<Async>,
		pub label: Lifetime,
		pub render_args: Option<SquareBrackets<Optimistic<Separated<RenderArg, Comma>>>>,
	}

	pub struct NotComma: PeekFrom, PopFrom {
		tt: TokenTree,
	}
}

impl DynOrParenDyn {
	pub fn span(&self) -> Span {
		match self {
			DynOrParenDyn::Dyn(r#dyn) => r#dyn.span(),
			DynOrParenDyn::ParenDyn(paren) => paren.span.join(),
		}
	}

	pub fn is_parenthesized(&self) -> bool {
		matches!(self, Self::ParenDyn(_))
	}
}

pub fn infer_pattern_type(_pattern: &Pattern) -> Option<TokenStream> {
	None //TODO
}

pub fn infer_expression_type(_expr: &Expression) -> Option<TokenStream> {
	None //TODO
}
