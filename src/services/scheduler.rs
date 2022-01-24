#![allow(missing_docs)] //TODO

use core::future::Future;

use crate::__dependency_injection::InjectionKey;

pub trait Scheduler {
	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()>>);
}

impl InjectionKey for dyn Scheduler {}

//TODO: This should be possible as `final` function in the extractable itself.
impl dyn Scheduler {
	/// Spaws a future on the scheduler.
	//TODO: Return some kind of task handle.
	pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		self.spawn_dynamic(Box::new(future));
	}
}

impl Scheduler for fn(Box<dyn Future<Output = ()>>) {
	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()>>) {
		self(future);
	}
}

impl Scheduler for Box<dyn Fn(Box<dyn Future<Output = ()>>)> {
	fn spawn_dynamic(&self, future: Box<dyn Future<Output = ()>>) {
		self(future);
	}
}
