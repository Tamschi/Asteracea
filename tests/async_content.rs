use asteracea::components::Suspense;

async fn future_text() -> String {
	"Hello Future!".to_string()
}

asteracea::component! { substrate =>
	Spinner()()

	[]
}

asteracea::component! { substrate =>
	async Async()()

	let self.text: String = future_text().await;
	!"{}"(self.text)
}

asteracea::component! { substrate =>
	pub Instant()() -> Sync

	<*Suspense
		'spinner: <*Spinner>
		'ready: async <*Async.await>
	>
}
