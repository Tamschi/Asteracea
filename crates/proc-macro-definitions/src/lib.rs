#![forbid(unsafe_code)]

use proc_macro::TokenStream as TokenStream1;

extern crate proc_macro;

mod components;
mod extract_dot_use;
mod lex;

#[proc_macro]
pub fn components(input: TokenStream1) -> TokenStream1 {
	components::components(input.into()).into()
}

#[proc_macro]
pub fn extract_dot_use(input: TokenStream1) -> TokenStream1 {
	extract_dot_use::extract_dot_use(input.into()).into()
}
