use super::render_callback::{RenderCallback, RenderMut, RenderOnce};
use crate::error::Escalation;
use crate::error::Result;
use bumpalo::Bump;
use futures_core::FusedFuture;
use lignin::{Node, ThreadSafety};
use std::{
	cell::RefCell,
	future::Future,
	ops::Deref,
	pin::Pin,
	ptr::NonNull,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Mutex, RwLock,
	},
	task::{Context, Poll},
};
use tiptoe::{Arc, IntrusivelyCountable, TipToe};

pub struct Async<Storage, F = Box<dyn Future<Output = Result<Storage>>>> {
	state: RwLock<AsyncState<Storage, F>>,
	handle: RefCell<Option<Arc<UntypedHandle>>>,
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
	Failed(Option<Escalation>),
}

enum AsyncStateProjectedMut<'proj, Storage, F> {
	Pending(Pin<&'proj mut F>),
	Ready(Pin<&'proj mut Storage>),
	Failed(&'proj mut Option<Escalation>),
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
			AsyncStateProjectedMut::Pending(future) => match future.poll(cx) {
				Poll::Ready(ready) => ready,
				Poll::Pending => return Poll::Pending,
			},

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

impl<Storage: 'static, F: 'static + Future<Output = Result<Storage>>> Async_ for Async<Storage, F> {
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

pub struct AsyncContent<'a, R: ?Sized + RenderCallback> {
	async_: Pin<&'a dyn Async_>,
	on_done: Box<R>,
}

impl<'a, R: ?Sized + RenderCallback> AsyncContent<'a, R> {
	fn synchronize(&mut self, anchor: &mut Option<AsyncContentSubscription>) -> Synchronized {
		match self.async_.synchronize(anchor) {
			Some(future) => Synchronized::Reset(future),
			None => Synchronized::Unchanged,
		}
	}
}

impl<'bump, S: ThreadSafety> AsyncContent<'_, RenderOnce<'_, 'bump, S>> {
	fn render(self, bump: &'bump Bump) -> Option<Result<Node<'bump, S>>> {
		todo!()
	}
}

impl<'bump, S: ThreadSafety> AsyncContent<'_, RenderMut<'_, 'bump, S>> {
	fn render(&mut self, bump: &'bump Bump) -> Option<Result<Node<'bump, S>>> {
		todo!()
	}
}

struct AsyncContentSubscription(Arc<UntypedHandle>);

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

pub enum Synchronized {
	Unchanged,
	Reset(ContentFuture),
}

pub struct ContentFuture(Arc<UntypedHandle>);

impl Future for ContentFuture {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		todo!()
	}
}
