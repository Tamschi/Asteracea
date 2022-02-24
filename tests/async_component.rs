async fn delayed() {}

asteracea::component! {
	pub async Async()() -> Sync

	let self._nothing: () = delayed().await;
	[]
}
