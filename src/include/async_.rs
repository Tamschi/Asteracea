//! Supporting implementation for `async` expressions in Asteracea templates.
//!
//! TODO: Does the [`Future`] still need some kind of multi-dispatch-wrapper?

use super::render_callback::{RenderCallback, RenderMut, RenderOnce};
use crate::error::{Caught, EscalateResult, Escalation, Result};
use bumpalo::Bump;
use futures_core::FusedFuture;
use lignin::{Guard, ThreadSafety};
use std::{
	any::Any,
	cell::RefCell,
	error::Error,
	fmt,
	fmt::{Display, Formatter},
	future::Future,
	ops::Deref,
	panic::AssertUnwindSafe,
	pin::Pin,
	ptr::NonNull,
	result::Result as stdResult,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Mutex, RwLock, RwLockReadGuard,
	},
	task::{Context, Poll},
};
use tiptoe::{Arc, IntrusivelyCountable, TipToe};

#[derive(Debug)]
struct FailedPreviouslyError;
impl Error for FailedPreviouslyError {}
impl Display for FailedPreviouslyError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str("`Async` had failed previously. Check earlier error.")
	}
}

/// Storage type for asynchronously initialised Asteracea template expressions.
pub struct Async<Storage, F = Pin<Box<dyn Send + Future<Output = Result<Storage>>>>> {
	state: RwLock<AsyncState<Storage, F>>,
	handle: RefCell<Option<Arc<UntypedHandle>>>,
}
impl<Storage: 'static, F: 'static + Send + Future<Output = Result<Storage>>> Async<Storage, F> {
	/// Creates a new instance of [`Async`] holding the given future.
	pub fn new(future_storage: F) -> Self {
		Self {
			state: AsyncState::Pending(future_storage).into(),
			handle: None.into(),
		}
	}

	/// Borrows this [`Async`] to pass it as content child to another component,
	/// along with the given render callback `on_done`.
	#[must_use]
	pub fn as_async_content<R: ?Sized + RenderCallback>(
		self: Pin<&Self>,
		on_done: Box<R>,
	) -> AsyncContent<'_, R> {
		AsyncContent {
			async_: self,
			on_done,
		}
	}

	pub fn storage_pinned<'a>(self: Pin<&'a Self>) -> Result<Pin<StorageGuard<'a, Storage, F>>> {
		let state = &self.get_ref().state;
		let read = state.read().unwrap();
		match &*read {
			AsyncState::Pending(_) => panic!(
				"Tried to get `asteracea::include::async::Async` storage before it was ready."
			),
			AsyncState::Ready(_) => return Ok(unsafe { Pin::new_unchecked(StorageGuard(read)) }),
			AsyncState::Failed(escalation) => match escalation {
				None => return Err(FailedPreviouslyError).escalate(),
				// Drop and re-lock:
				Some(_) => (),
			},
		}

		match &mut *self.state.write().unwrap() {
			AsyncState::Pending(_) | AsyncState::Ready(_) => unreachable!(),
			AsyncState::Failed(caught) => {
				if let Some(caught) = caught.take() {
					Err(caught).escalate()
				} else {
					Err(FailedPreviouslyError).escalate()
				}
			}
		}
	}
}

impl<Storage, F> Drop for Async<Storage, F> {
	fn drop(&mut self) {
		if let Some(handle) = self.handle.get_mut() {
			*handle.state.lock().unwrap() = None
		}
	}
}

/// Holds a reference to Asteracea expression storage for an [`Async`] that is ready.
pub struct StorageGuard<'a, Storage, F>(RwLockReadGuard<'a, AsyncState<Storage, F>>);
impl<Storage, F> Deref for StorageGuard<'_, Storage, F> {
	type Target = Storage;

	fn deref(&self) -> &Self::Target {
		match &*self.0 {
			AsyncState::Ready(storage) => storage,
			AsyncState::Pending(_) | AsyncState::Failed(_) => unreachable!(),
		}
	}
}

struct Dereferenceable<T: ?Sized>(NonNull<T>);
impl<T: ?Sized> Deref for Dereferenceable<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.as_ref() }
	}
}

struct UntypedHandle {
	counter: TipToe,
	/// Not actually static but always dereferenceable.
	state: Mutex<Option<Pin<Dereferenceable<dyn AsyncState_>>>>,
	subscribers: AtomicUsize,
}

unsafe impl IntrusivelyCountable for UntypedHandle {
	type RefCounter = TipToe;

	fn ref_counter(&self) -> &Self::RefCounter {
		&self.counter
	}
}

enum AsyncState<Storage, F> {
	Pending(F),
	Ready(Storage),
	Failed(Option<Caught<dyn Send + Any>>),
}

enum AsyncStateProjectedMut<'proj, Storage, F> {
	Pending(Pin<&'proj mut F>),
	Ready(Pin<&'proj mut Storage>),
	Failed(&'proj mut Option<Caught<dyn Send + Any>>),
}

impl<Storage, F> AsyncState<Storage, F> {
	fn project_mut(self: Pin<&mut Self>) -> AsyncStateProjectedMut<'_, Storage, F> {
		match unsafe { Pin::into_inner_unchecked(self) } {
			AsyncState::Pending(future) => {
				AsyncStateProjectedMut::Pending(unsafe { Pin::new_unchecked(future) })
			}
			AsyncState::Ready(ready) => {
				AsyncStateProjectedMut::Ready(unsafe { Pin::new_unchecked(ready) })
			}
			AsyncState::Failed(failed) => AsyncStateProjectedMut::Failed(failed),
		}
	}
}

trait PollTranspose {
	type Output;
	fn transpose(self) -> Self::Output;
}
impl<T, E> PollTranspose for Poll<stdResult<T, E>> {
	type Output = stdResult<Poll<T>, E>;

	fn transpose(self) -> Self::Output {
		Ok(match self {
			Poll::Ready(ready) => Poll::Ready(ready?),
			Poll::Pending => Poll::Pending,
		})
	}
}
impl<T, E> PollTranspose for stdResult<Poll<T>, E> {
	type Output = Poll<stdResult<T, E>>;

	fn transpose(self) -> Self::Output {
		match self {
			Ok(Poll::Ready(ok)) => Poll::Ready(Ok(ok)),
			Ok(Poll::Pending) => Poll::Pending,
			Err(error) => Poll::Ready(Err(error)),
		}
	}
}

impl<Storage, F: Future<Output = Result<Storage>>> AsyncState_ for RwLock<AsyncState<Storage, F>> {
	fn poll(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<()> {
		let mut write = match self.write() {
			Ok(write) => write,

			// The untyped future here is purely to run evaluation,
			// and if the underlying instance is poisoned that's not possible,
			// so we're done here.
			Err(_) => return Poll::Ready(()),
		};

		let result = match unsafe { Pin::new_unchecked(&mut *write) }.project_mut() {
			AsyncStateProjectedMut::Pending(future) => {
				match Escalation::catch_any(AssertUnwindSafe(
					//UNWIND-SAFETY:
					// The future is dropped a bit below, before it can be interacted with again.
					|| future.poll(cx).transpose(),
				))
				.transpose()
				{
					Poll::Ready(result) => result,
					Poll::Pending => return Poll::Pending,
				}
			}

			// Similarly here, we don't need to care how it completed, just *that* it completed.
			// (Any `Escalation` is re-thrown during rendering.)
			AsyncStateProjectedMut::Ready(_) | AsyncStateProjectedMut::Failed(_) => {
				return Poll::Ready(())
			}
		};

		*write = match result {
			Ok(storage) => AsyncState::Ready(storage),
			Err(escalation) => AsyncState::Failed(Some(escalation)),
		};
		Poll::Ready(())
	}

	fn is_done(&self) -> bool {
		match &*self.read().unwrap() {
			AsyncState::Pending(_) => false,
			AsyncState::Ready(_) | AsyncState::Failed(_) => true,
		}
	}
}

impl<Storage: 'static, F: 'static + Send + Future<Output = Result<Storage>>> Async_
	for Async<Storage, F>
{
	fn synchronize(
		self: Pin<&Self>,
		anchor: &mut Option<AsyncContentSubscription>,
	) -> Option<ContentFuture> {
		let mut handle = self.handle.borrow_mut();
		let handle = match &*self.state.read().unwrap() {
			AsyncState::Pending(_) => Some(handle.get_or_insert_with(|| {
				Arc::new(UntypedHandle {
					counter: TipToe::new(),
					state: Mutex::new(Some(unsafe {
						Pin::new_unchecked(Dereferenceable(NonNull::new_unchecked(
							&self.state as *const _ as *mut RwLock<AsyncState<Storage, F>>
								as *mut _,
						)))
					})),
					subscribers: 0.into(),
				})
			})),
			AsyncState::Ready(_) | AsyncState::Failed(_) => {
				drop(handle.take());
				None
			}
		};

		match (handle, anchor) {
			(None, None) => None,
			(None, anchor @ Some(_)) => {
				drop(anchor.take());
				None
			}
			(Some(a), Some(b)) if Arc::ptr_eq(a, &b.0) => None,
			(Some(handle), anchor) => {
				*anchor = Some(AsyncContentSubscription::new(Arc::clone(handle)));
				Some(ContentFuture(Arc::clone(handle)))
			}
		}
	}

	fn is_done(&self) -> bool {
		match &*self.state.read().unwrap() {
			AsyncState::Pending(_) => false,
			AsyncState::Ready(_) | AsyncState::Failed(_) => true,
		}
	}
}

impl<'a, Storage, F: Future<Output = Result<Storage>>> Future for Async<Storage, F> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		AsyncState_::poll(
			unsafe { self.into_ref().map_unchecked(|this| &this.state) },
			cx,
		)
	}
}

impl<Storage, F: Future<Output = Result<Storage>>> FusedFuture for Async<Storage, F> {
	fn is_terminated(&self) -> bool {
		let read = match self.state.read() {
			Ok(read) => read,
			Err(_) => return true,
		};
		match *read {
			AsyncState::Pending { .. } => false,
			AsyncState::Ready(_) | AsyncState::Failed(_) => true,
		}
	}
}

trait Async_ {
	fn synchronize(
		self: Pin<&Self>,
		anchor: &mut Option<AsyncContentSubscription>,
	) -> Option<ContentFuture>;
	fn is_done(&self) -> bool;
}

trait AsyncState_ {
	fn poll(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<()>;
	fn is_done(&self) -> bool;
}

impl Future for Pin<&dyn AsyncState_> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		AsyncState_::poll(*self, cx)
	}
}

/// Analogous to a [`RenderCallback`] passed as part of a content child argument.
pub struct AsyncContent<'a, R: ?Sized + RenderCallback> {
	async_: Pin<&'a dyn Async_>,
	on_done: Box<R>,
}

impl<'a, R: ?Sized + RenderCallback> AsyncContent<'a, R> {
	pub fn synchronize(&self, anchor: &mut Option<AsyncContentSubscription>) -> Synchronized {
		match self.async_.synchronize(anchor) {
			Some(future) => Synchronized::Reset(future),
			None => Synchronized::Unchanged,
		}
	}
}

impl<'bump, S: ThreadSafety> AsyncContent<'_, RenderOnce<'_, 'bump, S>> {
	pub fn render(self, bump: &'bump Bump) -> Option<Result<Guard<'bump, S>>> {
		self.async_.is_done().then(|| (self.on_done)(bump))
	}
}

impl<'bump, S: ThreadSafety> AsyncContent<'_, RenderMut<'_, 'bump, S>> {
	pub fn render(&mut self, bump: &'bump Bump) -> Option<Result<Guard<'bump, S>>> {
		self.async_.is_done().then(|| (self.on_done)(bump))
	}
}

/// Iff all of these are dropped, then the respective [`ContentFuture`]s are cancelled.
pub struct AsyncContentSubscription(Arc<UntypedHandle>);

impl AsyncContentSubscription {
	fn new(arc: Arc<UntypedHandle>) -> Self {
		arc.subscribers.fetch_add(1, Ordering::Relaxed);
		Self(arc)
	}
}

impl Drop for AsyncContentSubscription {
	fn drop(&mut self) {
		// This is simply a cancellation, so there's no data dependency here.
		self.0.subscribers.fetch_sub(1, Ordering::Relaxed);
	}
}

/// May hold a new [`ContentFuture`] to schedule.
#[must_use = "Asynchronous content children are only evaluated by polling their associated `ContentFuture`."]
pub enum Synchronized {
	Unchanged,
	Reset(ContentFuture),
}

/// Schedule to evaluate an async content child.
/// ('static + [`Unpin`] + [`Send`] + [`Future`] + [`FusedFuture`])

pub struct ContentFuture(Arc<UntypedHandle>);

/// # Safety Notes
///
/// > The tricky bit here is the [`Caught<dyn Send + Any>`](`Caught`) stored inside an [`RwLock`],
/// > which requires also [`Sync`] to be thread-safe.
/// >
/// > However, that instance is never shared (instead it is removed while a write lock is taken out),
/// > so implementing [`Send`] here *should* be fine.
unsafe impl Send for ContentFuture {}

impl Future for ContentFuture {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let handle = &*self.0;

		// It *may* be a good idea to make sure this happens before trying to take the lock.
		if handle.subscribers.load(Ordering::Acquire) == 0 {
			return Poll::Ready(());
		}

		match &*handle.state.lock().unwrap() {
			None => Poll::Ready(()),
			Some(state) => state.as_ref().poll(cx),
		}
	}
}

impl FusedFuture for ContentFuture {
	fn is_terminated(&self) -> bool {
		let handle = &*self.0;

		// It *may* be a good idea to make sure this happens before trying to take the lock.
		if handle.subscribers.load(Ordering::Acquire) == 0 {
			return true;
		}

		match &*handle.state.lock().unwrap() {
			None => true,
			Some(state) => state.as_ref().is_done(),
		}
	}
}
