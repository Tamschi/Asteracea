use std::{
	boxed,
	ops::ControlFlow::{self, Continue},
};

use loess::{
	Error, ErrorPriority, Errors, Input, PeekFrom, PopFrom, PopParsedFrom, grammar,
	quote_into_mixed_site,
	scaffold::{CurlyBraces, Greedy, Optimistic, Parentheses, SquareBrackets},
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
use proc_macro2::{Group, Ident, Span, TokenStream};

mod child;
mod let_field;

grammar! {
	pub struct Statement: PopFrom {
		attrs: Greedy<Vec<OuterAttribute>>,
		stmt: Statement_,
	}

	pub enum Statement_: PopFrom {
		ParenBrace(Parentheses<CurlyBraces>),
		BracketBrace(SquareBrackets<CurlyBraces>),
		For(ForLoop),
		ParenFor(ParenForLoop),
		BracketFor(BracketForLoop),
		Block(Block),
		Box(BoxStatement),
		Semi(Semi),
		Str(FormattedStr),
		Transclusion(Transclusion),
		LetField(let_field::LetField),
		Child(Optimistic<child::Child>),
	} else "Expected Asteracea statement.";
}

impl Statement {
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

		match self.stmt {
			Statement_::ParenBrace(Parentheses {
				span: paren_span,
				contents: CurlyBraces {
					span: braces_span,
					contents: rust,
				},
			}) => {
				// No fields.
				quote_into_mixed_site!(paren_span.join(), root, init_proc, {
					{#(self.attrs)}
					{#located_at(braces_span.join()) {
						{ {#(rust)} }
					}}
				})
				// Not rendered.
			}
			Statement_::BracketBrace(SquareBrackets {
				span: brackets_span,
				contents: CurlyBraces {
					span: braces_span,
					contents: rust,
				},
			}) => {
				// No fields.
				// No init.
				quote_into_mixed_site!(brackets_span.join(), root, render_proc, {
					{#(self.attrs)}
					<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
						&mut acc,
						ctx,
						{#located_at(braces_span.join()) {
							{ {#(rust)} }
						}}
					);
				})
			}
			Statement_::For(stmt) => todo!("Emit For."),
			Statement_::ParenFor(stmt) => todo!("Emit ParenFor."),
			Statement_::BracketFor(stmt) => stmt.emit(
				root,
				substrate,
				content_parent,
				&additional_field_attrs,
				fields,
				&self.attrs,
				init_proc,
				init_guards,
				render_proc,
			),
			Statement_::Block(stmt) => stmt.emit(
				root,
				substrate,
				content_parent,
				&additional_field_attrs,
				fields,
				&self.attrs,
				init_proc,
				init_guards,
				render_proc,
			),
			Statement_::Box(stmt) => todo!("Emit Box."),
			Statement_::Semi(_) => {
				// No fields.
				// No init.
				// Not rendered.
			}
			Statement_::Str(stmt) => todo!("Emit Str."),
			Statement_::Transclusion(stmt) => todo!("Emit Transclusion."),
			Statement_::Child(stmt) => stmt.emit(
				root,
				substrate,
				content_parent,
				&additional_field_attrs,
				fields,
				&self.attrs,
				init_proc,
				init_guards,
				render_proc,
			),
			Statement_::LetField(stmt) => todo!("Emit LetField."),
		}
	}
}

grammar! {
	pub struct ForLoop: PeekFrom, PopFrom {
		// pub outer_attributes: Greedy<OuterAttribute>,
		pub r#for: For,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: CurlyBraces<Vec<Statement>>,
	}

	pub struct ParenForLoop: PeekFrom, PopFrom {
		// pub outer_attributes: Greedy<OuterAttribute>,
		pub paren_for: Parentheses<For>,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: CurlyBraces<Vec<Statement>>,
	}

	pub struct BracketForLoop: PeekFrom, PopFrom {
		// pub outer_attributes: Greedy<OuterAttribute>,
		pub bracket_for: SquareBrackets<For>,
		pub pattern: Pattern,
		pub r#in: In,
		pub expression: ExpressionExceptStructExpression,
		pub block: Block,
	}

	pub struct Block: PeekFrom, PopFrom {
		pub block: CurlyBraces<Optimistic<Vec<Statement>>>,
	}

	pub struct BoxStatement: PeekFrom, PopFrom {
		pub r#box: Box,
		pub storage: Option<Storage>,
		pub statement: boxed::Box<Statement>,
	}

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

	/// Note: `format!` is not emitted if literal doesn't contain curly braces
	///        and `format_args` is [`None`].
	pub struct FormattedStr: PeekFrom, PopFrom {
		pub literal: AnyStringLiteral,
		pub format_args: Option<Parentheses>,
		pub semi: Semi,
	}

	pub struct Transclusion: PeekFrom, PopFrom {
		pub dot_dot: DotDot,
		pub expression: Expression,
		pub semi: Semi,
	}
}

impl Block {
	pub fn emit(
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
	) {
		let Block {
			block: CurlyBraces {
				span: block_span,
				contents,
			},
		} = self;
		let mut inner_init_proc = TokenStream::new();
		let mut inner_init_guards = vec![];
		let mut inner_render_proc = TokenStream::new();
		for statement in contents {
			statement.emit(
				root,
				substrate,
				content_parent,
				&additional_field_attrs,
				fields,
				&mut inner_init_proc,
				&mut inner_init_guards,
				&mut inner_render_proc,
			);
		}

		quote_into_mixed_site!(block_span.join(), root, init_proc, {
			{#for attr in attrs {
				{#(attr)}
			}}
			let {#({
				let guard = Identifier(Ident::new( &format!("init_guard_{}", init_guards.len()),block_span.join().resolved_at(Span::mixed_site())));
				init_guards.push(guard.clone());
				guard
			})} = {
				{#(inner_init_proc)}
				(
					{#for guard in inner_init_guards.into_iter().rev() {
						{#(guard)},
					}}
				)
			};
		});
		quote_into_mixed_site!(block_span.join(), root, render_proc, {
			{#for attr in attrs {
				{#(attr)}
			}}
			{
				{#(inner_render_proc)}
			}
		});
	}
}

impl BracketForLoop {
	pub fn emit(
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
	) {
		let BracketForLoop {
			bracket_for: SquareBrackets {
				span,
				contents: r#for,
			},
			pattern,
			r#in,
			expression,
			block,
		} = self;

		quote_into_mixed_site!(span.join(), root, render_proc, {
			{#for attr in attrs {
				{#(attr)}
			}}
			<{#(content_parent)} as {#root}::ContentParentOf<'r, {#(substrate)}, _>>::on_rendered(
				&mut acc,
				ctx,
				{
					let mut acc = <Self as {#root}::ContentParent<'r, {#(substrate)}>>::start_accumulation();
					{#(r#for, pattern, r#in, expression, {
						let mut render_proc = TokenStream::new();
						block.emit(
							root,
							substrate,
							content_parent,
							&additional_field_attrs,
							fields,
							&[],
							init_proc,
							init_guards,
							&mut render_proc,
						);
						render_proc
					})}
					<Self as {#root}::ContentParent<'r, {#(substrate)}>>::finalize_accumulation(acc, ctx)?
				}
			);
		});
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
