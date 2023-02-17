async fn delayed() {}

asteracea::component! { substrate =>
	async Child()()

	let self._nothing: () = delayed().await;
	[]
}

asteracea::component! { substrate =>
	pub async Parent()() -> Sync

	<*Child.await>
}
