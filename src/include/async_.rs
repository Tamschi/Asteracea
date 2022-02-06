use std::{
	future::Future,
	pin::Pin,
	sync::{Arc, Mutex, RwLock, Weak},
	task::{Context, Poll},
};

use futures_core::FusedFuture;

use crate::error::Escalation;

pub struct Async<'a, Storage, F = Box<dyn Future<Output = Result<Storage, Escalation>>>> {
	state: RwLock<AsyncState<'a, Storage, F>>,
}

type TypedHandle<'a, Storage, F> = Mutex<Option<Pin<&'a Async<'a, Storage, F>>>>;
type UntypedHandle<'a> = Mutex<Option<Pin<&'a dyn Async_>>>;

enum AsyncState<'a, Storage, F> {
	Pending {
		future: F,
		handle: Option<Arc<TypedHandle<'a, Storage, F>>>,
	},
	Ready(Storage),
	Failed(Option<Escalation>),
}

enum AsyncState_<'a, 'proj, Storage, F> {
	Pending {
		future: Pin<&'proj mut F>,
		handle: &'proj Option<Arc<TypedHandle<'a, Storage, F>>>,
	},
	Ready(Pin<&'proj mut Storage>),
	Failed(&'proj mut Option<Escalation>),
}

impl<'a, Storage, F> AsyncState<'a, Storage, F> {
	fn project_mut(self: Pin<&mut Self>) -> AsyncState_<'a, '_, Storage, F> {
		match unsafe { Pin::into_inner_unchecked(self) } {
			AsyncState::Pending { future, handle } => AsyncState_::Pending {
				future: unsafe { Pin::new_unchecked(future) },
				handle,
			},
			AsyncState::Ready(ready) => AsyncState_::Ready(unsafe { Pin::new_unchecked(ready) }),
			AsyncState::Failed(failed) => AsyncState_::Failed(failed),
		}
	}
}

impl<Storage, F: Future<Output = Result<Storage, Escalation>>> Async_ for Async<'_, Storage, F> {
	fn poll(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<()> {
		let write = match self.state.write() {
			Ok(write) => write,

			// The untyped future here is purely to run evaluation,
			// and if the underlying instance is poisoned that's not possible,
			// so we're done here.
			Err(_) => return Poll::Ready(()),
		};

		let result = match unsafe { Pin::new_unchecked(&mut *write) }.project_mut() {
			AsyncState_::Pending { future, handle } => match future.poll(cx) {
				Poll::Ready(ready) => ready,
				Poll::Pending => return Poll::Pending,
			},

			// Similarly here, we don't need to care how it completed, just *that* it completed.
			// (Any `Escalation` is re-thrown during rendering.)
			AsyncState_::Ready(_) | AsyncState_::Failed(_) => return Poll::Ready(()),
		};

		*write = match result {
			Ok(storage) => AsyncState::Ready(storage),
			Err(escalation) => AsyncState::Failed(Some(escalation)),
		};
		Poll::Ready(())
	}
}

impl<'a, Storage, F: Future<Output = Result<Storage, Escalation>>> Future
	for Async<'a, Storage, F>
{
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		Async_::poll(self.into_ref(), cx)
	}
}

impl<Storage, F: Future<Output = Result<Storage, Escalation>>> FusedFuture
	for Async<'_, Storage, F>
{
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
	fn poll(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<()>;
}

impl Future for Pin<&dyn Async_> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		Async_::poll(*self, cx)
	}
}

pub trait AsyncContent {
	fn synchronize(&mut self, anchor: &mut Option<AsyncContentAnchor>) -> Synchronized {}
}

pub enum Synchronized {
	Unchanged,
	Reset(ContentFuture),
}

pub struct ContentFuture {
	weak: Weak<Handle>,
}

impl Future for ContentFuture {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		todo!()
	}
}
