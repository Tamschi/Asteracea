#![allow(clippy::module_name_repetitions)]

use std::{
	any::Any,
	error::Error,
	fmt::Display,
	panic::{catch_unwind, UnwindSafe},
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

#[derive(Debug)]
#[allow(clippy::empty_enum)]
enum Impl {
	#[cfg(not(feature = "force-unwind"))]
	Error(Box<dyn Send + Any>),
}

impl Error for GUIError {}
impl Display for GUIError {
	#[allow(unused_variables)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		#[allow(clippy::match_single_binding)]
		match &self.0 {
			#[cfg(not(feature = "force-unwind"))]
			Impl::Error(boxed) => "GUIError".fmt(f), //TODO
			#[cfg(feature = "force-unwind")]
			_ => unreachable!(),
		}
	}
}

pub trait SendAnyError: Send + Any + Error {}
impl<E: Send + Any + Error> SendAnyError for E {}

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
		GUIError(Impl::Error(Box::new(self)))
	}
}

pub trait IntoGUIError2 {
	fn into_gui_error(self) -> GUIError;
}
impl IntoGUIError2 for &'static str {
	fn into_gui_error(self) -> GUIError {
		#[cfg(all(feature = "force-unwind", not(feature = "backtrace")))]
		//FIXME: Replace this with panic_any once that lands.
		std::panic::resume_unwind(Box::new(self));

		#[cfg(all(feature = "force-unwind", feature = "backtrace"))]
		panic!(format!("{:#}", self));

		#[cfg(not(feature = "force-unwind"))]
		GUIError(Impl::Error(Box::new(self)))
	}
}
impl IntoGUIError2 for String {
	fn into_gui_error(self) -> GUIError {
		#[cfg(all(feature = "force-unwind", not(feature = "backtrace")))]
		//FIXME: Replace this with panic_any once that lands.
		std::panic::resume_unwind(Box::new(self));

		#[cfg(all(feature = "force-unwind", feature = "backtrace"))]
		panic!(format!("{:#}", self));

		#[cfg(not(feature = "force-unwind"))]
		GUIError(Impl::Error(Box::new(self)))
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

impl GUIError {
	/// Catches any [`GUIError`] or, if possible, any (other) panic currently unwinding the stack.
	///
	/// # Errors
	///
	/// Iff a [`GUIError`] or panic is caught, it is returned in the [`Err`] variant.
	pub fn catch_any<F, T>(f: F) -> Result<T, Box<dyn Send + Any>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, GUIError>,
	{
		#[allow(clippy::match_same_arms)]
		match catch_unwind(f) {
			Ok(Ok(t)) => Ok(t),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => unreachable!(),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(GUIError(Impl::Error(e)))) => Err(e),
			Err(e) => Err(e),
		}
	}

	/// Catches [`GUIError`]s and, if possible, (other) panics currently unwinding the stack that are an `E`.
	///
	/// # Errors
	///
	/// Iff a [`GUIError`] or panic is caught and successfully downcast to `E`, it is returned in the [`Err`] variant.
	pub fn catch<F, T, E>(f: F) -> Result<Result<T, GUIError>, Box<E>>
	where
		F: UnwindSafe + FnOnce() -> Result<T, GUIError>,
		E: 'static,
	{
		let error = match catch_unwind(f) {
			Ok(Ok(t)) => return Ok(Ok(t)),
			#[cfg(feature = "force-unwind")]
			Ok(Err(_)) => unreachable!(),
			#[cfg(not(feature = "force-unwind"))]
			Ok(Err(GUIError(Impl::Error(e)))) => e,
			Err(e) => e,
		};
		match error.downcast() {
			Ok(e) => Err(e),
			Err(e) => {
				#[cfg(feature = "force-unwind")]
				std::panic::resume_unwind(e);
				#[cfg(not(feature = "force-unwind"))]
				Ok(Err(GUIError(Impl::Error(e))))
			}
		}
	}
}
