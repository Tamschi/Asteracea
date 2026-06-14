use std::boxed;

use loess::{
	grammar,
	scaffold::{CurlyBraces, Parentheses, SquareBrackets},
	Error, ErrorPriority, Errors, Input, IntoTokens, PeekFrom, PopFrom, PopParsedFrom,
};
use loess_rust::{
	ident::Identifier,
	lex::{
		keywords::{As, Box, For, In, SelfLowercase, Struct},
		token::{
			literal::AnyStringLiteral,
			punct::{Colon, Dot, DotDot, Semi},
		},
	},
	vis::Visibility,
};
use loess_rust_opaque::{
	Expression, ExpressionExceptStructExpression, Pattern, Statement as RustStatement,
};
use proc_macro2::TokenStream;

struct SkipPeek<T: ?Sized>(pub T);
impl<T> PeekFrom for SkipPeek<T> {
	fn peek_from(_input: &Input) -> bool {
		true
	}
}
impl<T: IntoTokens> IntoTokens for SkipPeek<T> {
	fn into_tokens(self, root: &TokenStream, tokens: &mut impl Extend<proc_macro2::TokenTree>) {
		self.0.into_tokens(root, tokens)
	}

	fn collect_tokens<TS: Default + Extend<proc_macro2::TokenTree>>(
		self,
		root: &TokenStream,
	) -> TS {
		self.0.collect_tokens(root)
	}
}
impl<T: PopFrom> PopParsedFrom for SkipPeek<T> {
	type Parsed = Self;

	fn pop_parsed_from(
		input: &mut Input,
		errors: &mut Errors,
	) -> Result<Self::Parsed, Option<Self::Parsed>> {
		Ok(Self(T::pop_from(input, errors).map_err(|_| None)?))
	}

	fn peek_pop_parsed_from(
		input: &mut Input,
		errors: &mut Errors,
	) -> Result<Option<Self::Parsed>, Option<Self::Parsed>>
	where
		Self: PeekFrom,
	{
		Ok(Some(Self::pop_from(input, errors)?))
	}
}

grammar! {
	pub enum Statement: PeekFrom, PopFrom, IntoTokens {
		ParenBrace(Parentheses<CurlyBraces<Vec<SkipPeek<RustStatement>>>>),
		BracketBrace(SquareBrackets<CurlyBraces<Vec<SkipPeek<RustStatement>>>>),
		For(ForLoop),
		ParenFor(ParenForLoop),
		Block(CurlyBraces<Vec<Statement>>),
		Box(BoxStatement),
		Semi(Semi),
		Str(FormattedStr),
		Transclusion(Transclusion),
		Child(child::Child),
	} else "Expected Asteracea statement.";
}

mod child;

grammar! {
	pub struct ForLoop: PeekFrom, PopFrom, IntoTokens {
		// pub outer_attributes: Greedy<OuterAttribute>,
		pub r#for: For,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: CurlyBraces<Vec<Statement>>,
	}

	pub struct ParenForLoop: PeekFrom, PopFrom, IntoTokens {
		// pub outer_attributes: Greedy<OuterAttribute>,
		pub paren_for: Parentheses<For>,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: CurlyBraces<Vec<Statement>>,
	}

	pub struct BoxStatement: PeekFrom, PopFrom, IntoTokens {
		pub r#box: Box,
		pub storage: Option<Storage>,
		pub statement: boxed::Box<Statement>,
	}

	pub struct Storage: PeekFrom, IntoTokens {
		pub r#as: As,
		pub visibility: Option<Visibility>,
		pub self_: SelfLowercase,
		pub dot: Dot,
		pub identifier: Identifier,
		pub storage_type: Option<StorageType>,
	}

	pub struct StorageType: PeekFrom, PopFrom, IntoTokens {
		pub colon: Colon,
		pub r#struct: Option<Struct>,
		pub identifier: Identifier,
	}

	/// Note: `format!` is not emitted if literal doesn't contain curly braces
	///        and `format_args` is [`None`].
	pub struct FormattedStr: PeekFrom, PopFrom, IntoTokens {
		pub literal: AnyStringLiteral,
		pub format_args: Option<Parentheses>,
		pub semi: Semi,
	}

	pub struct Transclusion: PeekFrom, PopFrom, IntoTokens {
		pub dot_dot: DotDot,
		pub expression: Expression,
		pub semi: Semi,
	}
}

impl PopParsedFrom for Storage {
	type Parsed = Self;

	fn pop_parsed_from(
		input: &mut Input,
		errors: &mut Errors,
	) -> Result<Self::Parsed, Option<Self::Parsed>> {
		let storage = Self {
			r#as: As::pop_from(input, errors).map_err(|_| None)?,
			visibility: Option::<Visibility>::pop_from(input, errors).map_err(|_| None)?,
			self_: SelfLowercase::pop_from(input, errors).map_err(|_| None)?,
			dot: Dot::pop_from(input, errors).map_err(|_| None)?,
			identifier: Identifier::pop_from(input, errors).map_err(|_| None)?,
			storage_type: Option::<StorageType>::pop_from(input, errors).map_err(|_| None)?,
		};

		if !CurlyBraces::<TokenStream>::peek_from(input) && !Semi::peek_from(input) {
			errors.push(Error::new(
				ErrorPriority::GRAMMAR,
				"Storage must be followed by `{` or `;`.",
				[input.front_span()],
			))
		}

		Ok(storage)
	}
}
