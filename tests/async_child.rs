async fn delayed() {}

asteracea::component! {
	async Child()()

	let self._nothing: () = delayed().await;
	[]
}

asteracea::component! {
	pub async Parent()() -> Sync

	<*Child.await>
}
