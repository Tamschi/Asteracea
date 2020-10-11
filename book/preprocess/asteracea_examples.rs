use std::{fmt::Write, iter};

use mdbook::{
	book::Book,
	errors::Result,
	preprocess::{Preprocessor, PreprocessorContext},
	BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::{cmark, State};

pub struct AsteraceaExamples;

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

				for event in Parser::new(&chapter.content).into_iter() {
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
								.flush(tag, &mut processed, &mut state)
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
	texts: Vec<CowStr<'a>>,
}

impl<'a> Default for CodeState<'a> {
	fn default() -> Self {
		Self {
			tags: vec![],
			texts: vec![],
		}
	}
}

impl<'a> CodeState<'a> {
	fn new(tag: CowStr) -> Option<Self> {
		let tags: Vec<_> = tag.split(',').collect();
		let total_count = tags.len();
		let tags: Vec<_> = tags
			.into_iter()
			.filter(|t| *t != "asteracea")
			.map(|t| t.to_owned())
			.collect();
		if tags.len() < total_count {
			Some(Self {
				tags,
				..CodeState::default()
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
		formatter: &mut impl Write,
		state: &mut Option<State<'static>>,
	) -> Result<()> {
		*state = cmark(
			iter::once(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
				self.tags.join(",").into(),
			))))
			.chain(self.texts.into_iter().map(Event::Text))
			.chain(iter::once(Event::End(Tag::CodeBlock(
				CodeBlockKind::Fenced(tag),
			)))),
			formatter,
			state.take(),
		)?
		.into();
		Ok(())
	}
}
