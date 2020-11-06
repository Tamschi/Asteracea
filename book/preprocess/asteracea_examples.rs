use line_col::LineColLookup;
use mdbook::{
	book::Book,
	errors::Result,
	preprocess::{Preprocessor, PreprocessorContext},
	BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::{cmark, State};
use quote::quote;
use std::{cell::RefCell, env, error::Error, fmt, fs::File, io::Write, iter, path::Path};

pub struct AsteraceaExamplesBuild {
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

	fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
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
										CodeBlockKind::Fenced(tag.replace(' ', ",").into()),
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
						Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(tag)))
							if code_state.is_none() =>
						{
							state = cmark(
								iter::once(Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(
									tag.replace(' ', ",").into(),
								)))),
								&mut processed,
								state,
							)
							.unwrap()
							.into()
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
	instantiate: CowStr<'a>,
	render: CowStr<'a>,
	texts: Vec<CowStr<'a>>,
}

impl<'a> CodeState<'a> {
	fn new(tag: CowStr) -> Option<Self> {
		let tags: Vec<_> = tag.split(' ').collect();
		let mut instantiate: Option<CowStr> = None;
		let mut render: CowStr = ".render(&mut bump)".into();
		let tags: Vec<_> = tags
			.into_iter()
			.filter(|t| {
				if t.starts_with("asteracea::render") {
					render = t
						.splitn(2, '=')
						.nth(1)
						.expect("Missing render call after asteracea::render")
						.into();
					false
				} else if t.starts_with("asteracea") {
					instantiate = Some(
						t.splitn(2, '=')
							.nth(1)
							.expect("Missing component name after asteracea")
							.into(),
					);
					false
				} else {
					true
				}
			})
			.map(|t| t.to_owned())
			.collect();
		if let Some(instantiate) = instantiate {
			let mut tags = tags;
			tags.push("no_run".into());
			tags.push("ro_playground".into());
			Some(Self {
				tags,
				instantiate: CowStr::Boxed(Box::new(instantiate.to_string()).into_boxed_str()),
				render: CowStr::Boxed(Box::new(render.to_string()).into_boxed_str()),
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
		_tag: CowStr<'a>,
		asteracea_html: &mut impl Write,
		key: &str,
	) -> Result<()> {
		writeln!(
			asteracea_html,
			r#""{key}" => {block}"#,
			key = key,
			block = quote! {{
				EXAMPLE_HERE

				let mut bump = asteracea::lignin_schema::lignin::bumpalo::Bump::new();
				let component = INSTANTIATE;
				let vdom = component RENDER;
				let mut html = String::new();
				lignin_html::render(&mut html, &vdom).debugless_unwrap();
				html
			}}
			.to_string()
			.replace("EXAMPLE_HERE", &self.texts.join(""))
			.replace("INSTANTIATE", self.instantiate.as_ref()) // TODO: Show the parametrisation somehow.
			.replace("RENDER", self.render.as_ref())
		)?;
		Ok(())
	}
}

fn keygen(chapter_name: &str, line_col: (usize, usize)) -> String {
	format!("{} L {} C {}", chapter_name, line_col.0, line_col.1)
}
