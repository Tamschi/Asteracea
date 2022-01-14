use mdbook::MDBook;
use std::error::Error;

mod asteracea_html {
	include!(concat!(env!("OUT_DIR"), "/asteracea_html.rs"));
}

mod preprocess;

fn main() -> Result<(), Box<dyn Error>> {
	build_book()?;
	Ok(())
}

fn build_book() -> Result<(), Box<dyn Error>> {
	let mut book = MDBook::load(".")?;
	book.with_preprocessor(preprocess::AsteraceaExamples)
		.build()?;
	Ok(())
}
