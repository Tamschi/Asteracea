asteracea::component! { substrate =>
	CoinFlip()(
		// 'heads(),
		// 'tails(),
		// '_()?,
	) []
}

asteracea::component! { substrate =>
	Random()()

	<*CoinFlip
		// 'heads: <div "Heads!">
		// 'tails: <div "Tails!">
		// <div "Edge!">
	>
}
