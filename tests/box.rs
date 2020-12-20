asteracea::component! {
	Countdown()(
		i: usize,
	)

	[
		!{i}
		spread if {i > 0} [
			"\n"
			defer box <*Countdown .i = {i - 1}>
		]
	]
}
