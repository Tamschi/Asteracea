use asteracea::{components::Suspense, substrates::web as substrate};

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
	pub Instant()()

	<*Suspense
		'spinner: <*Spinner>
		'ready: async <*Async.await>
	>
}
