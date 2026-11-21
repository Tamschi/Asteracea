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
		pub args: UpTo<DynOrBracketedDyn, Separated<InitArg, Comma>>,
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
		pub r#dyn: DynOrBracketedDyn,
		pub r#mut: Option<Mut>, //TODO: Can this be part of `pattern`?
		pub pattern: Pattern,
		pub question: Option<Question>,
		pub colon: Colon,
		pub r#type: Type,
		pub default: Option<(Eq, Expression)>,
	}

	pub enum DynOrBracketedDyn: PeekFrom, PopFrom {
		/// `init`-only
		Dyn(Dyn),
		/// Persisted, and bound only in `render`.
		BracketedDyn(SquareBrackets<Dyn>),
		//TODO: Variant for both? `{dyn}`?
	} else "Expected `dyn` or `[dyn]`.";

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

impl DynOrBracketedDyn {
	pub fn span(&self) -> Span {
		match self {
			DynOrBracketedDyn::Dyn(r#dyn) => r#dyn.span(),
			DynOrBracketedDyn::BracketedDyn(paren) => paren.span.join(),
		}
	}

	pub fn is_bracketed(&self) -> bool {
		matches!(self, Self::BracketedDyn(_))
	}
}

pub fn infer_pattern_type(_pattern: &Pattern) -> Option<TokenStream> {
	None //TODO
}

pub fn infer_expression_type(_expr: &Expression) -> Option<TokenStream> {
	None //TODO
}
