use asteracea::substrates::web as substrate;

async fn delayed() {}

asteracea::component! { substrate =>
	pub async Async()()

	let self._nothing: () = delayed().await;
	[]
}
