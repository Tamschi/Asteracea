use core::future::Future;
use rhizome_crate::extractable;

extractable! {
	pub abstract trait Scheduler

	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()> >);
}

//TODO: This should be possible as `final` function in the extractable itself.
impl dyn Scheduler {
	pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		self.spawn_dynamic(Box::new(future))
	}
}

impl Scheduler for fn(Box<dyn Future<Output = ()>>) {
	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()>>) {
		self(future)
	}
}

impl Scheduler for Box<dyn Fn(Box<dyn Future<Output = ()>>)> {
	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()>>) {
		self(future)
	}
}
