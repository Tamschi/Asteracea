use std::ops::ControlFlow::{self, Continue};

use loess::{
	Error, ErrorPriority, Errors, Input, PeekFrom, PopFrom, PopParsedFrom, grammar,
	quote_into_mixed_site,
	scaffold::{CurlyBraces, Greedy, Optimistic},
};
use loess_rust::{
	attributes::OuterAttribute,
	ident::Identifier,
	lex::{
		keywords::{As, SelfLowercase, Struct},
		token::punct::{Colon, Dot, Semi},
	},
	vis::Visibility,
};
use proc_macro2::{Group, Span, TokenStream};

use crate::components::component::statements::rust_blocks::{BracketBrace, ParenBrace};

mod block;
mod boxed;
mod child;
mod r#for;
mod r#if;
mod let_field;
mod r#match;
mod rust_blocks;
mod semi;
mod text;
mod transclusion;
mod r#while;

grammar! {
	pub struct Statement: PopFrom {
		attrs: Greedy<Vec<OuterAttribute>>,
		stmt: std::boxed::Box<dyn StatementImpl>,
	}

	pub enum Statement_: PopFrom {
		ParenBrace(ParenBrace),
		BracketBrace(BracketBrace),
		For(r#for::CachedFor),
		ParenFor(r#for::ParenFor),
		BracketFor(r#for::BracketFor),
		BracketIf(r#if::BracketIf),
		BracketMatch(r#match::BracketMatch),
		BracketWhile(r#while::BracketWhile),
		Block(block::Block),
		Box(boxed::Box),
		Semi(semi::Semi),
		Text(text::Text),
		Transclusion(transclusion::Transclusion),
		LetField(let_field::LetField),
		Child(Optimistic<child::Child>),
	} else "Expected Asteracea statement.";
}

trait StatementImpl {
	fn hint_size(
		&self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		attrs: &[OuterAttribute],
		tokens: &mut TokenStream,
	);

	fn boxed_emit(
		self: Box<Self>,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	);

	fn emit(
		self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		attrs: &[OuterAttribute],
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	);
}

impl dyn StatementImpl {
	fn hint_size_unknown(span: Span, root: &TokenStream, tokens: &mut TokenStream) {
		quote_into_mixed_site!(span, root, tokens, {
			{#root}::SizeHint::UNKNOWN,
		});
	}

	fn hint_size_zero(span: Span, root: &TokenStream, tokens: &mut TokenStream) {
		quote_into_mixed_site!(span, root, tokens, {
			{#root}::SizeHint::ZERO,
		});
	}

	fn hint_size_one(span: Span, root: &TokenStream, tokens: &mut TokenStream) {
		quote_into_mixed_site!(span, root, tokens, {
			{#root}::SizeHint::ONE,
		});
	}
}

impl PopParsedFrom for Box<dyn StatementImpl> {
	type Parsed = Self;

	fn pop_parsed_from(
		input: &mut Input,
		errors: &mut Errors,
	) -> ControlFlow<Option<Self::Parsed>, Option<Self::Parsed>> {
		use std::boxed::Box;

		let stmt = Statement_::pop_from(input, errors);
		(match stmt {
			ControlFlow::Continue(_) => ControlFlow::Continue,
			ControlFlow::Break(_) => ControlFlow::Break,
		})(
			match stmt {
				Continue(stmt) | ControlFlow::Break(stmt) => stmt,
			}
			.map(|stmt| match stmt {
				Statement_::ParenBrace(stmt) => Box::new(stmt) as Box<dyn StatementImpl>,
				Statement_::BracketBrace(stmt) => Box::new(stmt),
				Statement_::For(stmt) => Box::new(stmt),
				Statement_::ParenFor(stmt) => Box::new(stmt),
				Statement_::BracketFor(stmt) => Box::new(stmt),
				Statement_::BracketIf(stmt) => Box::new(stmt),
				Statement_::BracketMatch(stmt) => Box::new(stmt),
				Statement_::BracketWhile(stmt) => Box::new(stmt),
				Statement_::Block(stmt) => Box::new(stmt),
				Statement_::Box(stmt) => Box::new(stmt),
				Statement_::Semi(stmt) => Box::new(stmt),
				Statement_::Text(stmt) => Box::new(stmt),
				Statement_::Transclusion(stmt) => Box::new(stmt),
				Statement_::LetField(stmt) => Box::new(stmt),
				Statement_::Child(stmt) => Box::new(stmt),
			}),
		)
	}
}

impl Statement {
	pub fn hint_size(
		&self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		tokens: &mut TokenStream,
	) {
		self.stmt
			.hint_size(root, substrate, content_parent, &self.attrs, tokens);
	}

	pub fn emit(
		self,
		root: &TokenStream,
		substrate: &Group,
		content_parent: &TokenStream,
		additional_field_attrs: &[&OuterAttribute],
		fields: &mut Vec<(Vec<OuterAttribute>, Identifier, TokenStream)>,
		init_proc: &mut TokenStream,
		init_guards: &mut Vec<Identifier>,
		render_proc: &mut TokenStream,
	) {
		let additional_field_attrs = additional_field_attrs
			.into_iter()
			.copied()
			.chain(&self.attrs)
			.collect::<Vec<_>>();

		self.stmt.boxed_emit(
			root,
			substrate,
			content_parent,
			&additional_field_attrs,
			fields,
			&self.attrs,
			init_proc,
			init_guards,
			render_proc,
		);
	}
}

grammar! {
	pub struct Storage: PeekFrom {
		pub r#as: As,
		pub visibility: Option<Visibility>,
		pub self_: SelfLowercase,
		pub dot: Dot,
		pub identifier: Identifier,
		pub storage_type: Option<StorageType>,
	}

	pub struct StorageType: PeekFrom, PopFrom {
		pub colon: Colon,
		pub r#struct: Option<Struct>,
		pub identifier: Identifier,
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
