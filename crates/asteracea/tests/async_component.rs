mod modname {
	#![allow(deprecated, dead_code)]

	pub(crate) async fn delayed() {}

	asteracea::component! {
		pub async Async()() -> Sync

		let self._nothing: () = delayed().await;
		[]
	}
}

async fn delayed() {}

asteracea::components! {
	pub async Async -> web {
		({ delayed().await; })
	}
}
