#![allow(clippy::module_name_repetitions)]

use std::{
	any::Any,
	borrow::Cow,
	error::Error,
	fmt::{self, Debug, Display, Formatter},
	panic::{catch_unwind, resume_unwind, UnwindSafe},
	writeln,
};

/// An error propagated along the component tree.
///
/// # About GUI Errors
///
/// GUI errors (including dependency resolution errors) are, at least in Asteracea, considered to be programming errors and not part of the expected control flow. As such, they are strongly deprioritised for optimisation and any built-in error handling primitives are a variation of 'fail once, fail forever' regarding their operands.
///
/// What this means in practice is that the framework may substitute panics for any [`Err(Escalation)`](`Err`) variant and therefore make all `new` and `render` methods on components effectively infallible. Additionally, panics unwound through the GUI are considered to be GUI errors and caught by Asteracea's error handling expressions.
///
/// *This is largely transparent to application code*, with two exceptions:
///
/// - Errors escalated with [`?`](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) on-GUI must be `Send + Any + Error` (or already a `Escalation`), and **iff** Asteracea is **forced** to substitute panics in a `panic = "abort"` environment, such an escalation will immediately abort the process.
/// - To handle [`Escalation`]s reliably, you **must** use [`Escalation::catch_any`]! This is done automatically by built-in and generated error handlers.
///
/// > The error escalation strategy is determined at compile-time. This will eventually become automatic via [`#[cfg(panic = "…")]`](https://github.com/rust-lang/rust/pull/74754), but until that is available on stable, may have to be chosen via the `"force-unwind"` feature. (When in doubt, enable the feature where you can.)
///
/// > Unwinding notably isn't supported on `wasm32-unknown-unknown` as of Rust 1.49. This means any builds targeting the web natively will have to use implicit explicit GUI error escalation for now.
///
/// For expected errors and errors raised off-GUI (incl. in event handlers), [please see the book for recoverable error handling strategies.](`TODO`)
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct Escalation(Impl);

impl<E: ?Sized> Caught<E> {
	#[allow(
		non_snake_case,
		unused_mut,
		unused_variables,
		clippy::needless_pass_by_value
	)]
	#[doc(hidden)]
	pub fn __Asteracea__with_traced_frame(mut self, frame: Cow<'static, str>) -> Self {
		if let Some(trace) = &mut self.trace {
			trace.push(frame)
		} else {
			self.trace = Some(vec![frame])
		}
		self
	}
}

impl From<Caught<dyn Send + Any>> for Escalation {
	fn from(caught: Caught<dyn Send + Any>) -> Self {
		let throwable = Throwable {
			source: caught.boxed,
			trace: caught.trace.unwrap_or_else(Vec::new),
		};
		if cfg!(feature = "force-unwind") || caught.was_panic {
			resume_unwind(Box::new(throwable))
		} else {
			#[cfg(not(feature = "force-unwind"))]
			return Escalation(Impl::Extant(throwable));
			{
				#![allow(unreachable_code)]
				unreachable!()
			}
		}
	}
}

impl<E: Send + Any> From<Caught<E>> for Escalation {
	fn from(caught: Caught<E>) -> Self {
		let throwable = Throwable {
			source: caught.boxed,
			trace: caught.trace.unwrap_or_else(Vec::new),
		};
		if cfg!(feature = "force-unwind") || caught.was_panic {
			resume_unwind(Box::new(throwable))
		} else {
			#[cfg(not(feature = "force-unwind"))]
			return Escalation(Impl::Extant(throwable));
			{
				#![allow(unreachable_code)]
				unreachable!()
			}
		}
	}
}

#[allow(dead_code)]
struct ErrorWrapper(Box<dyn SendAnyErrorCasting>);

#[derive(Debug)]
struct Throwable {
	source: Box<dyn Send + Any>,
	trace: Vec<Cow<'static, str>>,
}

#[derive(Debug)]
#[allow(clippy::empty_enum)]
enum Impl {
	#[cfg(not(feature = "force-unwind"))]
	Extant(Throwable),
}

//TODO: This *probably* needs some clean-up.
pub trait SendAnyError: Send + Any + Error {}
impl<E: Send + Any + Error> SendAnyError for E {}
trait SendAnyErrorCasting: SendAnyError {
	fn as_any(&self) -> &'_ (dyn Any + '_);
	fn into_any_box(self: Box<Self>) -> Box<dyn Any>;
	fn into_any_send_box(self: Box<Self>) -> Box<dyn Send + Any>;
}
impl<E: SendAnyError> SendAnyErrorCasting for E {
	fn as_any(&self) -> &'_ (dyn Any + '_) {
		self
	}

	fn into_any_box(self: Box<Self>) -> Box<dyn Any> {
		self
	}

	fn into_any_send_box(self: Box<Self>) -> Box<dyn Send + Any> {
		self
	}
}

pub trait Escalate {
	type Output;
	fn escalate(self) -> Self::Output;
}
impl<E: SendAnyError> Escalate for E {
	type Output = Escalation;
	fn escalate(self) -> Self::Output {
		let throwable = Throwable {
			source: Box::new(ErrorWrapper(Box::new(self))),
			trace: vec![],
		};
		if cfg!(feature = "force-unwind") {
			//FIXME: Replace this with panic_any once that lands.
			std::panic::resume_unwind(Box::new(throwable));
		} else {
			#[cfg(not(feature = "force-unwind"))]
			return Escalation(Impl::Extant(throwable));
			{
				#![allow(unreachable_code)]
				unreachable!()
			}
		}
	}
}

pub trait EscalateResult {
	type Output;
	fn escalate(self) -> Self::Output;
}
impl<Ok, E: Escalate> EscalateResult for Result<Ok, E> {
	type Output = Result<Ok, E::Output>;

	fn escalate(self) -> Self::Output {
		self.map_err(|e| e.escalate())
	}
}

/// A caught [`Escalation`], which may have originated as error or panic.
///
/// Re-escalating this type always panics if it was created from a panic, in order to presever unwind-safety-related errors.
///
/// Panics resumed from this type (including via tracing instrumentation with `"backtrace"` enabled) are wrapped to enable tracing if that was not the case before.
/// This is transparent towards the `Escalation::catch…` functions and other APIs inside this module, but may affect error handlers from other crates.
#[must_use = "Please ignore caught escalations explicitly with `let _ =` if this is intentional."]
pub struct Caught<E: ?Sized> {
	// An error or panic.
	boxed: Box<E>,
	trace: Option<Vec<Cow<'static, str>>>,
	was_panic: bool,
}
impl<E: ?Sized> Caught<E> {
	#[must_use]
	pub fn into_boxed(self) -> Box<E> {
		self.boxed
	}
}
impl<E> Caught<E> {
	#[must_use]
	pub fn into_inner(self) -> E {
		*self.boxed
	}
}
impl Debug for Caught<dyn Send + Any> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(str) = self.boxed.downcast_ref::<&str>() {
			Display::fmt(str, f)?
		} else if let Some(string) = self.boxed.downcast_ref::<String>() {
			Display::fmt(string, f)?
		} else if let Some(wrapper) = self.boxed.downcast_ref::<ErrorWrapper>() {
			Display::fmt(&wrapper.0, f)?
		} else {
			writeln!(f, "type ID: {:?}", self.boxed.type_id())?
		}
		writeln!(f)?;
		writeln!(f)?;
		for frame in self.trace.iter().flatten() {
			writeln!(f, "in {}", frame)?
		}
		Ok(())
	}
}
impl<E: Debug> Debug for Caught<E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.boxed.fmt(f)?;
		writeln!(f)?;
		writeln!(f)?;
		for frame in self.trace.iter().flatten() {
			writeln!(f, "in {}", frame)?
		}
		Ok(())
	}
}
impl<E: Display> Display for Caught<E> {
	#[allow(unused_variables)]
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.boxed.fmt(f)?;
		writeln!(f)?;
		writeln!(f)?;
		for frame in self.trace.iter().flatten() {
			writeln!(f, "in {}", frame)?
		}
		Ok(())
	}
}
impl<E: Error> Error for Caught<E> {
	fn cause(&self) -> Option<&dyn Error> {
		Some(&self.boxed)
	}
}

impl Escalation {
	/// Catches any [`Escalation`] currently unwinding the stack.
	///
	/// Plain panics are considered to also be escalations,
	/// and re-escalating them always leads to instrumentation for tracing.
	///
	/// # Errors
	///
	/// Iff an [`Escalation`] is caught, it is returned in the [`Err`] variant.
	pub fn catch_any<F, T>(f: F) -> Result<T, Caught<dyn Send + Any>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, Escalation>,
	{
		#[allow(clippy::match_same_arms)]
		match catch_unwind(f) {
			Ok(Ok(t)) => Ok(t),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => Err(()).expect("unreachable"),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(Escalation(Impl::Extant(Throwable { source, trace })))) => Err(Caught {
				boxed: source,
				trace: Some(trace),
				was_panic: false,
			}),
			Err(panic) => Err(match Box::<dyn Send + Any>::downcast::<Throwable>(panic) {
				Ok(thrown) => Caught {
					boxed: thrown.source,
					trace: Some(thrown.trace),
					was_panic: true,
				},
				Err(panic) => Caught {
					boxed: panic,
					trace: None,
					was_panic: true,
				},
			}),
		}
	}

	/// Catches [`Escalation`]s and, if possible, (other) panics currently unwinding the stack that are an `E`.
	///
	/// Even if not caught, panics are converted into [`GuiError`]s (which may re-panic them).
	///
	/// # Errors
	///
	/// Iff a [`Escalation`] or panic is caught and successfully downcast to `E`, it is returned in the [`Err`] variant.
	pub fn catch<F, T, E: Error>(f: F) -> Result<Result<T, Escalation>, Caught<E>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, Escalation>,
		E: 'static,
	{
		let (thrown, was_panic) = match catch_unwind(f) {
			Ok(Ok(t)) => return Ok(Ok(t)),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => Err(()).expect("unreachable"),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(Escalation(Impl::Extant(thrown)))) => (thrown, false),
			Err(panic) => match Box::<dyn Send + Any>::downcast::<Throwable>(panic) {
				Ok(thrown) => (*thrown, true),
				Err(panic) => {
					// Not instrumented.
					match Box::<dyn Send + Any>::downcast(panic) {
						Ok(e) => {
							return Err(Caught {
								boxed: e,
								trace: None,
								was_panic: true,
							})
						}
						Err(panic) => resume_unwind(panic),
					}
				}
			},
		};
		let uncaught = match Box::<dyn Send + Any>::downcast::<ErrorWrapper>(thrown.source) {
			Ok(wrapper) => match Box::<dyn Send + Any>::downcast::<E>(wrapper) {
				Ok(caught) => {
					return Err(Caught {
						boxed: caught,
						trace: Some(thrown.trace),
						was_panic,
					})
				}
				Err(wrapper) => wrapper,
			},
			Err(other) => match Box::<dyn Send + Any>::downcast(other) {
				Ok(e) => {
					return Err(Caught {
						boxed: e,
						trace: Some(thrown.trace),
						was_panic,
					})
				}
				Err(uncaught) => uncaught,
			},
		};
		let throwable = Throwable {
			source: uncaught,
			trace: thrown.trace,
		};
		if cfg!(feature = "force-unwind") || was_panic {
			resume_unwind(Box::new(throwable))
		} else {
			#[cfg(not(feature = "force-unwind"))]
			{
				return Ok(Err(Escalation(Impl::Extant(throwable))));
			}
			{
				#![allow(unreachable_code)]
				Err(()).expect(
					"Workaround for clippy::missing_panic_docs. Only reachable if the \"force-unwind\" feature is both active and not active.",
				)
			}
		}
	}
}
