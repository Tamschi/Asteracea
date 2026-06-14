use loess::{
	grammar, quote_into_with_exact_span,
	scaffold::{CurlyBraces, Parentheses, SquareBrackets},
	IntoTokens, SimpleSpanned,
};
use loess_rust::{
	ident::Identifier,
	lex::{
		keywords::{Async, Const},
		token::punct::RArrow,
	},
	vis::Visibility,
};
use proc_macro2::{Ident, TokenStream, TokenTree};
use statements::Statement;

pub mod statements;

grammar! {
	pub struct Component: PopFrom {
		pub visibility: Option<Visibility>,
		pub r#const: Option<Const>,
		pub r#async: Option<Async>,
		pub name: Identifier,
		pub constructor_args: Option<Parentheses>,
		pub render_args: Option<SquareBrackets>,
		pub r_arrow: RArrow,
		pub substrate: Identifier,
		pub body: CurlyBraces<Vec<Statement>>,
	}
}

impl IntoTokens for Component {
	fn into_tokens(self, root: &TokenStream, tokens: &mut impl Extend<TokenTree>) {
		let Self {
			visibility,
			r#const,
			r#async,
			name,
			constructor_args,
			render_args,
			r_arrow,
			substrate,
			body,
		} = self;

		// This (hopefully) enables unused function warnings.
		let new = Ident::new("new", name.span());
		let render = Ident::new("render", name.span());

		quote_into_with_exact_span!(name.span(), root, tokens, {
			{#(&visibility)} struct {#(&name)} {}

			{#mixed_site {
				impl {#root}::Component<{#(&substrate)}> for {#(name)} {
					//TODO: Fallible initialisation.
					fn {#(new)}(
						parent_node: {#(&substrate)}::ParentNode,
						args: <Self as {#root}::Component<{#(&substrate)}>>::NewArgs,
					) -> impl {#root}::pinned_init::PinInit<Self> {
						let init = |_pointer| todo!();

						unsafe { {#root}::pinned_init::init_from_closure(init) }
					}
				}
			}}
		});
	}
}
