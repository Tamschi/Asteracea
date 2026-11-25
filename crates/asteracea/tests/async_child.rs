mod old {
	#![allow(deprecated, dead_code)]

	pub(crate) async fn delayed() {}

	asteracea::component! {
		async Child()()

		let self._nothing: () = delayed().await;
		[]
	}

	asteracea::component! {
		pub async Parent()() -> Sync

		<*Child.await>
	}
}

pub async fn delayed() {}

asteracea::components! {
	async Child -> web {
		({ delayed().await; })
	}

	pub async Parent -> web {
		Child.await;
	}
}
