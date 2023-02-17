use asteracea::substrates::web as substrate;

async fn delayed() {}

asteracea::component! { substrate =>
	async Child()()

	let self._nothing: () = delayed().await;
	[]
}

asteracea::component! { substrate =>
	pub async Parent()()

	<*Child.await>
}
