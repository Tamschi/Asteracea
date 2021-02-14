asteracea::component! {
	Any(
		pub sometimes*?: usize,
		pub some/many+?: usize,
		pub one/any*: usize,
		pub always+: usize,
	)()

	new with {}

	[]
}

asteracea::component! {
	Ones()()

	[]
}