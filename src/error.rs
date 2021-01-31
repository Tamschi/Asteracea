#![allow(clippy::module_name_repetitions)]

use std::{
	any::Any,
	borrow::Cow,
	error::Error,
	fmt::{self, Debug, Display, Formatter},
	panic::{catch_unwind, UnwindSafe},
	writeln,
};

/// An error propagated along the component tree.
///
/// # About GUI Errors
///
/// GUI errors (including dependency resolution errors) are, at least in Asteracea, considered to be programming errors and not part of the expected control flow. As such, they are strongly deprioritised for optimisation and any built-in error handling primitives are a variation of 'fail once, fail forever' regarding their operands.
///
/// What this means in practice is that the framework may substitute panics for any [`Err(GUIError)`](`Err`) variant and therefore make all `new` and `render` methods on components effectively infallible. Additionally, panics unwound through the GUI are considered to be GUI errors and caught by Asteracea's error handling expressions.
///
/// *This is largely transparent to application code*, with two exceptions:
///
/// - Errors escalated with [`?`](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) on-GUI must be `Send + Any + Error` (or already a `GUIError`), and **iff** Asteracea is **forced** to substitute panics in a `panic = "abort"` environment, such an escalation will immediately abort the process.
/// - To handle [`GUIError`]s reliably, you **must** use [`GUIError::catch_any`]! This is done automatically by built-in and generated error handlers.
///
/// > The error escalation strategy is determined at compile-time. This will eventually become automatic via [`#[cfg(panic = "…")]`](https://github.com/rust-lang/rust/pull/74754), but until that is available on stable, may have to be chosen via the `"force-unwind"` feature. (When in doubt, enable the feature where you can.)
///
/// > Unwinding notably isn't supported on `wasm32-unknown-unknown` as of Rust 1.49. This means any builds targeting the web natively will have to use implicit explicit GUI error escalation for now.
///
/// For expected errors and errors raised off-GUI (incl. in event handlers), [please see the book for recoverable error handling strategies.](`TODO`)
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct GUIError(Impl);

impl GUIError {
	#[allow(
		non_snake_case,
		unused_mut,
		unused_variables,
		clippy::needless_pass_by_value
	)]
	#[doc(hidden)]
	#[must_use]
	pub fn __Asteracea__with_traced_frame(mut self, frame: Cow<'static, str>) -> Self {
		#[cfg(not(feature = "force-unwind"))]
		{
			let GUIError(Impl::Error { trace, .. }) = &mut self;
			trace.push(frame);
		}
		self
	}
}

#[allow(dead_code)]
struct ErrorWrapper(Box<dyn SendAnyErrorCasting>);

#[derive(Debug)]
#[allow(clippy::empty_enum)]
enum Impl {
	#[cfg(not(feature = "force-unwind"))]
	Error {
		error: Box<dyn Send + Any>,
		trace: Vec<Cow<'static, str>>,
	},
}

impl Error for GUIError {}
impl Display for GUIError {
	#[allow(unused_variables)]
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		#[allow(clippy::match_single_binding)]
		match &self.0 {
			#[cfg(not(feature = "force-unwind"))]
			Impl::Error { error, trace } => {
				if let Some(str) = error.downcast_ref::<&str>() {
					Display::fmt(str, f)?
				} else if let Some(string) = error.downcast_ref::<String>() {
					Display::fmt(string, f)?
				} else {
					writeln!(f, "GUIError(type ID: {:?})", error.type_id())?
				}
				writeln!(f)?;
				for frame in trace {
					writeln!(f, "in {}", frame)?
				}
				Ok(())
			}
			#[cfg(feature = "force-unwind")]
			_ => unreachable!(),
		}
	}
}

//TODO: This *probably* needs some clean-up.
pub trait SendAnyError: Send + Any + Error {}
impl<E: Send + Any + Error> SendAnyError for E {}
trait SendAnyErrorCasting: SendAnyError {
	fn as_any(&self) -> &'_ (dyn Any + '_);
	fn into_any_box(self: Box<Self>) -> Box<dyn Any>;
}
impl<E: SendAnyError> SendAnyErrorCasting for E {
	fn as_any(&self) -> &'_ (dyn Any + '_) {
		self
	}

	fn into_any_box(self: Box<Self>) -> Box<dyn Any> {
		self
	}
}

pub trait IntoGUIError {
	fn into_gui_error(self) -> GUIError;
}
impl<E: SendAnyError> IntoGUIError for E {
	fn into_gui_error(self) -> GUIError {
		#[cfg(all(feature = "force-unwind", not(feature = "backtrace")))]
		//FIXME: Replace this with panic_any once that lands.
		std::panic::resume_unwind(Box::new(self));

		#[cfg(all(feature = "force-unwind", feature = "backtrace"))]
		panic!(format!("{:#}", self));

		#[cfg(not(feature = "force-unwind"))]
		GUIError(Impl::Error {
			error: Box::new(ErrorWrapper(Box::new(self))),
			trace: Vec::new(),
		})
	}
}

//TODO: Is this actually necessary?
// Does this need to be public?
pub trait IntoGUIError2 {
	fn into_gui_error(self) -> GUIError;
}
impl IntoGUIError2 for &'static str {
	fn into_gui_error(self) -> GUIError {
		#[cfg(all(feature = "force-unwind", not(feature = "backtrace")))]
		//FIXME: Replace this with panic_any once that lands.
		std::panic::resume_unwind(Box::new(self));

		#[cfg(all(feature = "force-unwind", feature = "backtrace"))]
		panic!(self.to_string());

		#[cfg(not(feature = "force-unwind"))]
		GUIError(Impl::Error {
			error: Box::new(self),
			trace: Vec::new(),
		})
	}
}
impl IntoGUIError2 for String {
	fn into_gui_error(self) -> GUIError {
		#[cfg(all(feature = "force-unwind", not(feature = "backtrace")))]
		//FIXME: Replace this with panic_any once that lands.
		std::panic::resume_unwind(Box::new(self));

		#[cfg(all(feature = "force-unwind", feature = "backtrace"))]
		panic!(self);

		#[cfg(not(feature = "force-unwind"))]
		GUIError(Impl::Error {
			error: Box::new(self),
			trace: Vec::new(),
		})
	}
}

pub trait IntoGUIResult {
	type Ok;
	/// Converts a given value (usually a `Result`) into a `Result<_, GUIError>`
	///
	/// # Errors
	///
	/// If `self` represents an error.
	fn into_gui_result(self) -> Result<Self::Ok, GUIError>;
}
impl<Ok, E: IntoGUIError> IntoGUIResult for Result<Ok, E> {
	type Ok = Ok;

	fn into_gui_result(self) -> Result<Ok, GUIError> {
		self.map_err(|e| e.into_gui_error())
	}
}

#[must_use = "Please ignore caught GUI errors explicitly with `let _ =` if this is intentional."]
pub struct Caught<E: ?Sized> {
	// An error or panic.
	boxed: Box<E>,
	#[cfg(not(feature = "force-unwind"))]
	trace: Option<Vec<Cow<'static, str>>>,
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
		#[cfg(not(feature = "force-unwind"))]
		writeln!(f)?;
		#[cfg(not(feature = "force-unwind"))]
		for frame in self.trace.iter().flatten() {
			writeln!(f, "in {}", frame)?
		}
		Ok(())
	}
}
impl<E: Debug> Debug for Caught<E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.boxed.fmt(f)?;
		#[cfg(not(feature = "force-unwind"))]
		{
			writeln!(f)?;
			for frame in self.trace.iter().flatten() {
				writeln!(f, "in {}", frame)?
			}
		}
		Ok(())
	}
}
impl<E: Display> Display for Caught<E> {
	#[allow(unused_variables)]
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.boxed.fmt(f)?;
		#[cfg(not(feature = "force-unwind"))]
		{
			writeln!(f)?;
			for frame in self.trace.iter().flatten() {
				writeln!(f, "in {}", frame)?
			}
		}
		Ok(())
	}
}
impl<E: Error> Error for Caught<E> {
	fn cause(&self) -> Option<&dyn Error> {
		Some(&self.boxed)
	}
}

impl GUIError {
	/// Catches any [`GUIError`] or, if possible, any (other) panic currently unwinding the stack.
	///
	/// # Errors
	///
	/// Iff a [`GUIError`] or panic is caught, it is returned in the [`Err`] variant.
	pub fn catch_any<F, T>(f: F) -> Result<T, Caught<dyn Send + Any>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, GUIError>,
	{
		#[allow(clippy::match_same_arms)]
		match catch_unwind(f) {
			Ok(Ok(t)) => Ok(t),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => unreachable!(),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(GUIError(Impl::Error { error, trace }))) => Err(Caught {
				boxed: error,
				trace: Some(trace),
			}),
			Err(panic) => Err(Caught {
				boxed: panic,
				#[cfg(not(feature = "force-unwind"))]
				trace: None,
			}),
		}
	}

	/// Catches [`GUIError`]s and, if possible, (other) panics currently unwinding the stack that are an `E`.
	///
	/// Even if not caught, panics are converted into [`GuiError`]s (which may re-panic them).
	///
	/// # Errors
	///
	/// Iff a [`GUIError`] or panic is caught and successfully downcast to `E`, it is returned in the [`Err`] variant.
	pub fn catch<F, T, E: Error>(f: F) -> Result<Result<T, GUIError>, Caught<E>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, GUIError>,
		E: 'static,
	{
		let caught = match catch_unwind(f) {
			Ok(Ok(t)) => return Ok(Ok(t)),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => unreachable!(),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(GUIError(Impl::Error { error, trace }))) => Caught {
				boxed: error,
				trace: Some(trace),
			},
			Err(panic) => Caught {
				boxed: panic,
				#[cfg(not(feature = "force-unwind"))]
				trace: None,
			},
		};
		match caught.boxed.downcast() {
			Ok(boxed) => Err(Caught {
				boxed,
				#[cfg(not(feature = "force-unwind"))]
				trace: caught.trace,
			}),
			Err(boxed) => match boxed.downcast::<ErrorWrapper>() {
				Err(boxed) => {
					#[cfg(feature = "force-unwind")]
					std::panic::resume_unwind(boxed);
					#[cfg(not(feature = "force-unwind"))]
					Ok(Err(GUIError(Impl::Error {
						error: boxed,
						trace: caught.trace.unwrap_or_else(Vec::new),
					})))
				}
				Ok(wrapper) => {
					let can_catch = Any::downcast_ref::<E>(wrapper.0.as_any()).is_some();
					if can_catch {
						Err(Caught {
							boxed: Box::<dyn Any>::downcast(wrapper.0.into_any_box()).unwrap(),
							#[cfg(not(feature = "force-unwind"))]
							trace: caught.trace,
						})
					} else {
						#[cfg(feature = "force-unwind")]
						std::panic::resume_unwind(Box::new(ErrorWrapper(wrapper.0)));
						#[cfg(not(feature = "force-unwind"))]
						Ok(Err(GUIError(Impl::Error {
							error: Box::new(ErrorWrapper(wrapper.0)),
							trace: caught.trace.unwrap_or_else(Vec::new),
						})))
					}
				}
			},
		}
	}
}
