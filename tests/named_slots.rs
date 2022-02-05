asteracea::component! {
	CoinFlip()(
		// 'heads(),
		// 'tails(),
		// '_()?,
	) []
}

asteracea::component! {
	Random()()

	<*CoinFlip
		// 'heads: <div "Heads!">
		// 'tails: <div "Tails!">
		// <div "Edge!">
	>
}