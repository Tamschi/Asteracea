async fn delayed() {}

asteracea::component! { substrate =>
	pub async Async()() -> Sync

	let self._nothing: () = delayed().await;
	[]
}
