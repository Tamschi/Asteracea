use asteracea::{components::Suspense, include::async_::ContentFuture, services::ContentRuntime};
use bumpalo::Bump;
use futures_core::Future;
use lignin_html::render_fragment;
use rhizome::sync::{Inject, Node};
use std::{
	any::TypeId,
	pin::Pin,
	sync::Mutex,
	task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use tap::Pipe;
use this_is_fine::FineExt;

async fn future_text() -> String {
	"Like a record!".to_string()
}

asteracea::component! {
	Spinner()()

	"Spinning right 'round…"
}

asteracea::component! {
	async Async()()

	let self.text: String = future_text().await;
	!"{}"(self.text)
}

asteracea::component! {
	Instant()() -> Sync

	<*Suspense
		'spinner: <*Spinner>
		'ready: async <*Async.await>
	>
}

#[test]
fn suspense() {
	let root = Node::new(TypeId::of::<()>());

	let future: Mutex<Option<ContentFuture>> = Mutex::default();
	let future: &Mutex<Option<ContentFuture>> = unsafe { &*(&future as *const _) }; // Needs to be `'static`.

	<dyn ContentRuntime>::inject(root.as_ref(), move |content_future| {
		if future.lock().unwrap().replace(content_future).is_some() {
			panic!("`ContentFuture` scheduled repeatedly!")
		}
	})
	.not_fine()
	.map_err(|_| ())
	.unwrap();

	let app = Instant::new(root.as_ref(), Instant::new_args_builder().build()).unwrap();
	let app = unsafe { Pin::new_unchecked(&app) };

	let mut bump = Bump::new();

	{
		let vdom = app
			.render(&bump, Instant::render_args_builder().build())
			.unwrap();

		let mut fragment = String::new();
		render_fragment(&vdom, &mut fragment, 1).unwrap();

		assert_eq!(fragment, "Spinning right 'round…");
	}

	bump.reset();

	{
		let vdom = app
			.render(&bump, Instant::render_args_builder().build())
			.unwrap();

		let mut fragment = String::new();
		render_fragment(&vdom, &mut fragment, 1).unwrap();

		assert_eq!(fragment, "Spinning right 'round…");
	}

	bump.reset();

	future
		.lock()
		.unwrap()
		.as_mut()
		.unwrap()
		.pipe(Pin::new)
		.poll(&mut Context::from_waker(&fake_waker()))
		.pipe(|poll| match poll {
			Poll::Ready(()) => (),
			Poll::Pending => panic!(),
		});

	{
		let vdom = app
			.render(&bump, Instant::render_args_builder().build())
			.unwrap();

		let mut fragment = String::new();
		render_fragment(&vdom, &mut fragment, 1).unwrap();

		assert_eq!(fragment, "Like a record!");
	}
}

fn fake_waker() -> Waker {
	const V_TABLE: RawWakerVTable =
		RawWakerVTable::new(|_| panic!(), |_| panic!(), |_| panic!(), |_| ());

	unsafe { Waker::from_raw(RawWaker::new(&() as *const _, &V_TABLE)) }
}
