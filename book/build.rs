use line_col::LineColLookup;
use mdbook::MDBook;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use std::{
	env,
	error::Error,
	fs::File,
	io::{Read, Write},
	ops::Range,
	path::Path,
};
use walkdir::WalkDir;

mod preprocess;

mod asteracea_html {
	pub fn get_html(_key: &str) -> Result<String, asteracea::error::Escalation> {
		unimplemented!("This is only available when running the built book")
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=src");

	build_book()?;
	generate_tests()?;
	Ok(())
}

fn build_book() -> Result<(), Box<dyn Error>> {
	let mut book = MDBook::load(".")?;
	book.with_preprocessor(preprocess::AsteraceaExamplesBuild::new()?)
		.build()?;
	Ok(())
}

fn generate_tests() -> Result<(), Box<dyn Error>> {
	println!(r#"cargo:rerun-if-changed="tests""#);

	let out_dir = Path::new(&env::var_os("OUT_DIR").ok_or("Missing OUT_DIR.")?).to_owned();

	let mut lib = File::create(out_dir.join("lib.rs"))?;

	let entries: Result<Vec<_>, _> = WalkDir::new("src").into_iter().collect();
	for entry in entries?.into_iter() {
		println!(r#"cargo:rerun-if-changed="{}""#, entry.path().display());
		if !entry.file_type().is_file()
			|| entry.path().extension().and_then(|e| e.to_str()) != Some("md")
		{
			continue;
		}

		let name_base = entry
			.path()
			.with_extension("")
			.display()
			.to_string()
			.replace(['/', '\\', '-'], "_")
			+ "_L";

		let mut text = String::new();
		File::open(entry.path())?.read_to_string(&mut text)?;
		let line_col = LineColLookup::new(&text);

		let mut file: Option<File> = None;
		for event in Parser::new(&text).into_offset_iter() {
			match event {
				(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tag))), Range { start, .. }) => {
					let line = line_col.get(start).0;
					let test_name = name_base.clone() + &line.to_string();

					writeln!(lib, "#[allow(non_snake_case)] mod {};", test_name)?;

					let test_path = Path::new(&out_dir).join(&test_name).with_extension("rs");
					file = File::create(&test_path)?.into();
					writeln!(file.as_mut().unwrap(), "//! ```{}", tag)?;
				}
				(Event::End(Tag::CodeBlock(_)), _) => {
					let mut file = file.take().unwrap();
					writeln!(file, "//! ```")?;
					file.flush()?;
				}
				(Event::Text(text), _) => {
					if let Some(file) = &mut file {
						for line in text.lines() {
							writeln!(file, "//! {}", line)?;
						}
					}
				}
				_ => (),
			}
		}
	}

	Ok(())
}
