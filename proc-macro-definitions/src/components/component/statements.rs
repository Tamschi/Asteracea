use std::{boxed, ops::ControlFlow::{self, Continue}};

use loess::{
	grammar,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
	Error, ErrorPriority, Errors, Input, PeekFrom, PopFrom, PopParsedFrom,
};
use loess_rust::{
	attributes::OuterAttribute,
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
use loess_rust_opaque::{Expression, ExpressionExceptStructExpression, Pattern};
use proc_macro2::TokenStream;

mod child;
mod let_field;

grammar! {
	pub struct Statement: PopFrom, IntoTokens {
		attrs: Greedy<Vec<OuterAttribute>>,
		stmt: Statement_,
	}

	pub enum Statement_: PopFrom, IntoTokens {
		ParenBrace(Parentheses<CurlyBraces>),
		BracketBrace(SquareBrackets<CurlyBraces>),
		For(ForLoop),
		ParenFor(ParenForLoop),
		Block(CurlyBraces<Optimistic<Vec<Statement>>>),
		Box(BoxStatement),
		Semi(Semi),
		Str(FormattedStr),
		Transclusion(Transclusion),
		Child(child::Child),
		LetField(let_field::LetField),
	} else "Expected Asteracea statement.";
}

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
	) -> ControlFlow<Option<Self::Parsed>, Option<Self::Parsed>> {
		let storage = As::pop_from(input, errors)
			.map_break(|_| None)?
			.zip(Option::<Visibility>::pop_from(input, errors).map_break(|_| None)?)
			.zip(SelfLowercase::pop_from(input, errors).map_break(|_| None)?)
			.zip(Dot::pop_from(input, errors).map_break(|_| None)?)
			.zip(Identifier::pop_from(input, errors).map_break(|_| None)?)
			.zip(Option::<StorageType>::pop_from(input, errors).map_break(|_| None)?)
			.map(
				|(((((r#as, visibility), self_), dot), identifier), storage_type)| Self {
					r#as,
					visibility,
					self_,
					dot,
					identifier,
					storage_type,
				},
			);

		if !CurlyBraces::<TokenStream>::peek_from(input) && !Semi::peek_from(input) {
			errors.push(Error::new(
				ErrorPriority::GRAMMAR,
				"Storage must be followed by `{` or `;`.",
				[input.front_span()],
			))
		}

		Continue(storage)
	}
}
