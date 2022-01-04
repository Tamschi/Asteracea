use crate::{
	error::Escalation,
	__::{AnonymousContentParentParameters, Built},
};
use ::std::pin::Pin;
use bumpalo::Bump;
use lignin::{Node, ThreadSafe};
use std::{any::type_name, cell::Cell, sync::Arc};
use typed_builder::TypedBuilder;

// A simple page router.
pub struct Router;

const _: () = {
	impl Router {
		pub fn new(
			_parent_node: &Arc<rhizome::Node>,
			RouterNewArgs {}: RouterNewArgs,
		) -> Result<Self, Escalation> {
			Ok(Self)
		}
		pub fn render<'bump>(
			self: Pin<&Self>,
			bump: &'bump Bump,
			RouterRenderArgs {
				__Asteracea__anonymous_content,
				path,
				rest,
			}: RouterRenderArgs<'_, 'bump>,
		) -> Result<Node<'bump, ThreadSafe>, Escalation> {
			for route in __Asteracea__anonymous_content {
				let (RouterParentParameters { paths }, render_content) = route;
				for route in paths {
					let wildcard_route = route.ends_with("/*");
					let route = if wildcard_route {
						&route[..route.len() - '*'.len_utf8()]
					} else {
						route
					};
					if (!wildcard_route && path == route)
						|| (wildcard_route && &path[..route.len()] == route)
					{
						if let Some(rest) = rest {
							let prev_rest = rest
								.replace(&path[route.strip_suffix('/').unwrap_or(route).len()..]);
							let content = render_content(bump);
							rest.set(prev_rest);
							return content;
						} else {
							return render_content(bump);
						}
					}
				}
			}
			todo!("Router default (explicit named child?).")
		}

		pub fn new_args_builder() -> RouterNewArgsBuilder<()> {
			RouterNewArgs::builder()
		}

		pub fn render_args_builder<'RENDER, 'bump: 'RENDER>(
		) -> RouterRenderArgsBuilder<'RENDER, 'bump> {
			RouterRenderArgsBuilder {
				rest: None,
				__Asteracea__anonymous_content: vec![],
			}
		}
	}

	#[derive(TypedBuilder)]
	pub struct RouterNewArgs {}

	pub struct RouterRenderArgsBuilder<'RENDER, 'bump: 'RENDER> {
		/// FIXME: Should statically work only once.
		rest: Option<&'RENDER Cell<&'bump str>>,
		__Asteracea__anonymous_content: Vec<(
			RouterParentParameters<'RENDER>,
			Box<dyn 'RENDER + FnOnce(&'bump Bump) -> Result<Node<'bump, ThreadSafe>, Escalation>>,
		)>,
	}

	impl<'RENDER, 'bump: 'RENDER> RouterRenderArgsBuilder<'RENDER, 'bump> {
		pub fn path(self, path: &'bump str) -> RouterRenderArgs<'RENDER, 'bump> {
			let Self {
				rest,
				__Asteracea__anonymous_content,
			} = self;
			RouterRenderArgs {
				path,
				rest,
				__Asteracea__anonymous_content,
			}
		}

		/// FIXME: Should statically work only once.
		pub fn rest(mut self, rest: &'RENDER Cell<&'bump str>) -> Self {
			self.rest
				.replace(rest)
				.ok_or(())
				.expect_err(&format!("Set `.rest` twice on `{}`", type_name::<Router>()));
			self
		}

		pub fn __Asteracea__anonymous_content(
			mut self,
			route: (
				RouterParentParameters<'RENDER>,
				Box<
					dyn 'RENDER
						+ FnOnce(&'bump Bump) -> Result<Node<'bump, ThreadSafe>, Escalation>,
				>,
			),
		) -> Self {
			self.__Asteracea__anonymous_content.push(route);
			self
		}
	}

	pub struct RouterRenderArgs<'RENDER, 'bump: 'RENDER> {
		//FIXME: Should be statically required.
		path: &'bump str,
		rest: Option<&'RENDER Cell<&'bump str>>,
		__Asteracea__anonymous_content: Vec<(
			RouterParentParameters<'RENDER>,
			Box<dyn 'RENDER + FnOnce(&'bump Bump) -> Result<Node<'bump, ThreadSafe>, Escalation>>,
		)>,
	}

	impl<'RENDER, 'bump: 'RENDER> RouterRenderArgs<'RENDER, 'bump> {
		pub fn build(self) -> Self {
			self
		}

		/// FIXME: Should statically work only once.
		pub fn rest(mut self, rest: &'RENDER Cell<&'bump str>) -> Self {
			self.rest
				.replace(rest)
				.ok_or(())
				.expect_err(&format!("Set `.rest` twice on `{}`", type_name::<Router>()));
			self
		}

		pub fn __Asteracea__anonymous_content(
			mut self,
			route: (
				RouterParentParameters<'RENDER>,
				Box<
					dyn 'RENDER
						+ FnOnce(&'bump Bump) -> Result<Node<'bump, ThreadSafe>, Escalation>,
				>,
			),
		) -> Self {
			self.__Asteracea__anonymous_content.push(route);
			self
		}
	}

	pub struct RouterParentParameters<'RENDER> {
		/// FIXME: This is inefficient.
		paths: Vec<&'RENDER str>,
	}

	impl Built for RouterParentParameters<'_> {
		type Builder = Self;

		fn builder() -> Self::Builder {
			Self { paths: vec![] }
		}
	}

	impl<'a> RouterParentParameters<'a> {
		pub fn build(self) -> Self {
			self
		}

		pub fn path(self, path: &'a str) -> RouterParentParameters<'a> {
			let Self { mut paths } = self;
			paths.push(path);
			RouterParentParameters { paths }
		}
	}
};
