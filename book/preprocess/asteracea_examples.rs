use line_col::LineColLookup;
use mdbook::{
	book::Book,
	errors::Result,
	preprocess::{Preprocessor, PreprocessorContext},
	BookItem,
};
use proc_macro2::{Ident, Span};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::{cmark, State};
use quote::quote;
use std::{
	cell::RefCell,
	env,
	error::Error,
	fmt,
	fs::File,
	io::Write,
	iter,
	path::{Path, PathBuf},
};

pub struct AsteraceaExamplesBuild {
	out_dir: PathBuf,
	asteracea_html: RefCell<File>,
}
pub struct AsteraceaExamples;

impl AsteraceaExamplesBuild {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let out_dir = Path::new(&env::var_os("OUT_DIR").ok_or("Missing OUT_DIR.")?).to_owned();
		Ok(Self {
			asteracea_html: {
				let mut file = File::create(out_dir.join("asteracea_html.rs"))?;
				writeln!(
					file,
					"{} {{ {} {{",
					quote! {
						use debugless_unwrap::DebuglessUnwrap as _;
						use asteracea::lignin_schema::lignin::Node;
						use asteracea::lignin_schema::lignin::bumpalo::Bump;
						use std::collections::HashMap;

						pub fn get_html(key: &str) -> String
					},
					quote! {
						match key
					}
				)?;
				file.into()
			},
			out_dir,
		})
	}
}

impl Drop for AsteraceaExamplesBuild {
	fn drop(&mut self) {
		let asteracea_html = &mut *self.asteracea_html.borrow_mut();
		writeln!(
			asteracea_html,
			"{} }}}}",
			quote!(other => panic!("Unknown key \"{}\"", other),)
		)
		.unwrap();
		asteracea_html.flush().unwrap();
	}
}

impl Preprocessor for AsteraceaExamplesBuild {
	fn name(&self) -> &str {
		"Asteracea Example (Build)"
	}

	fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
		book.for_each_mut(|item| {
			if let BookItem::Chapter(chapter) = item {
				let mut code_state: Option<CodeState> = None;
				let chapter_name = chapter.name.as_str();

				let line_col = LineColLookup::new(&chapter.content);

				for (event, offset) in Parser::new(&chapter.content).into_offset_iter() {
					match event {
						Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tag))) => {
							code_state = CodeState::new(tag.clone())
						}
						Event::Text(text) if code_state.is_some() => {
							code_state.as_mut().unwrap().add_text(text)
						}
						Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(tag)))
							if code_state.is_some() =>
						{
							code_state
								.take()
								.unwrap()
								.flush_build(
									tag,
									self.out_dir.as_path(),
									&mut *self.asteracea_html.borrow_mut(),
									&keygen(chapter_name, line_col.get(offset.start)),
								)
								.unwrap()
						}
						_ => (),
					}
				}
			}
		});
		Ok(book)
	}
}

impl Preprocessor for AsteraceaExamples {
	fn name(&self) -> &str {
		"Asteracea Examples"
	}

	fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
		book.for_each_mut(|item| {
			if let BookItem::Chapter(chapter) = item {
				let mut state: Option<State> = None;
				let mut processed = String::new();

				let mut code_state: Option<CodeState> = None;

				let chapter_name = chapter.name.as_str();
				let line_col = LineColLookup::new(&chapter.content);

				for (event, offset) in Parser::new(&chapter.content).into_offset_iter() {
					match event {
						Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tag))) => {
							code_state = CodeState::new(tag.clone());
							if code_state.is_none() {
								state = cmark(
									iter::once(Event::Start(Tag::CodeBlock(
										CodeBlockKind::Fenced(tag),
									))),
									&mut processed,
									state,
								)
								.unwrap()
								.into()
							}
						}
						Event::Text(text) if code_state.is_some() => {
							code_state.as_mut().unwrap().add_text(text)
						}
						Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(tag)))
							if code_state.is_some() =>
						{
							code_state
								.take()
								.unwrap()
								.flush(
									tag,
									&mut processed,
									&mut state,
									chapter_name,
									line_col.get(offset.start),
								)
								.unwrap()
						}
						event => {
							state = cmark(iter::once(event), &mut processed, state)
								.unwrap()
								.into()
						}
					}
				}

				chapter.content = processed;
			}
		});
		Ok(book)
	}
}

struct CodeState<'a> {
	tags: Vec<String>,
	component_ident: Ident,
	texts: Vec<CowStr<'a>>,
}

impl<'a> CodeState<'a> {
	fn new(tag: CowStr) -> Option<Self> {
		let tags: Vec<_> = tag.split(',').collect();
		let mut component_ident: Option<Ident> = None;
		let tags: Vec<_> = tags
			.into_iter()
			.filter(|t| {
				if t.starts_with("asteracea") {
					component_ident = Ident::new(
						t.splitn(2, '=')
							.nth(1)
							.expect("Missing component name after asteracea"),
						Span::call_site(),
					)
					.into();
					false
				} else {
					true
				}
			})
			.map(|t| t.to_owned())
			.collect();
		if let Some(component_ident) = component_ident {
			Some(Self {
				tags,
				component_ident,
				texts: Vec::new(),
			})
		} else {
			None
		}
	}

	fn add_text(&mut self, text: CowStr<'a>) {
		self.texts.push(text)
	}

	fn flush(
		self,
		tag: CowStr<'a>,
		formatter: &mut impl fmt::Write,
		state: &mut Option<State<'static>>,
		chapter_name: &str,
		line_col: (usize, usize),
	) -> Result<()> {
		*state = cmark(
			iter::once(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
				self.tags.join(",").into(),
			))))
			.chain(self.texts.into_iter().map(Event::Text))
			.chain(iter::once(Event::End(Tag::CodeBlock(
				CodeBlockKind::Fenced(tag),
			))))
			.chain(
				vec![
					Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("html".into()))),
					Event::Text(
						crate::asteracea_html::get_html(&keygen(chapter_name, line_col)).into(),
					),
					Event::Text("\n".into()),
					Event::End(Tag::CodeBlock(CodeBlockKind::Fenced("html".into()))),
				]
				.into_iter(),
			),
			formatter,
			state.take(),
		)?
		.into();
		Ok(())
	}

	fn flush_build(
		self,
		tag: CowStr<'a>,
		out_dir: &Path,
		asteracea_html: &mut impl Write,
		key: &str,
	) -> Result<()> {
		let c_ident = self.component_ident;
		writeln!(
			asteracea_html,
			r#""{key}" => {block}"#,
			key = key,
			block = quote! {{
				EXAMPLE_HERE

				let mut bump = asteracea::lignin_schema::lignin::bumpalo::Bump::new();
				let vdom = #c_ident::new().render(&mut bump);
				let mut html = String::new();
				lignin_html::render(&mut html, &vdom).debugless_unwrap();
				html
			}}
			.to_string()
			.replace("EXAMPLE_HERE", &self.texts.join(""))
		)?;
		Ok(())
	}
}

fn keygen(chapter_name: &str, line_col: (usize, usize)) -> String {
	format!("{} L {} C {}", chapter_name, line_col.0, line_col.1)
}
