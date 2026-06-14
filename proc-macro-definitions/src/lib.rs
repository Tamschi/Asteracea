#![forbid(unsafe_code)]

use proc_macro::TokenStream as TokenStream1;

extern crate proc_macro;

mod components;

#[proc_macro]
pub fn components(input: TokenStream1) -> TokenStream1 {
	components::components(input.into()).into()
}
