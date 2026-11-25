mod _sub;

asteracea::components! {
	#![substrate = _sub::Sub]

	CoinFlip{
		'heads,
		'tails,
		'_?,
	} => ;

	Random => CoinFlip{
		div{ "Edge!"; }
		'heads: div{ "Heads!"; }
		'tails: div{ "Tails!"; }
	>}
}
