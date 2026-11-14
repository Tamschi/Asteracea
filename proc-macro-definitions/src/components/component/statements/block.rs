use loess::grammar;
use loess::quote_into_mixed_site;
use loess::scaffold::CurlyBraces;
use loess::scaffold::Optimistic;
use loess_rust::attributes::OuterAttribute;
use loess_rust::ident::Identifier;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::{Group, Ident};

use crate::components::component::statements::Statement;
use crate::components::component::statements::StatementImpl;

grammar! {
	pub struct Block: PeekFrom, PopFrom {
		pub block: CurlyBraces<Optimistic<Vec<Statement>>>,
	}
}

impl StatementImpl for Block {
	fn hint_size(
		&self,
		root: &TokenStream,
		_substrate: &Group,
		_content_parent: &TokenStream,
		_attrs: &[OuterAttribute], //TODO: Account for `cfg`s.
		tokens: &mut TokenStream,
	) {
		<dyn StatementImpl>::hint_size_one(self.block.span.join(), root, tokens);
	}

	fn boxed_emit(
		self: std::boxed::Box<Self>,
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
		self.emit(
			root,
			substrate,
			content_parent,
			additional_field_attrs,
			fields,
			attrs,
			init_proc,
			init_guards,
			render_proc,
		);
	}

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
				let guard = Identifier(Ident::new( &format!("init_guard_{}", init_guards.len()), block_span.join().resolved_at(Span::mixed_site())));
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
