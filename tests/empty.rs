#![allow(dead_code)]

#[allow(deprecated)]
mod old {
	asteracea::component! {
		Empty()()

		[]
	}
}

asteracea::components! {
	#![substrate = web]

	Empty => ;
	pub Empty2 => ;
	pub(crate) Empty3 => ;
	Empty4() => ;
	Empty5[] => ;
	Empty6()[] => ;
	const Empty7 => ;
	async Empty8 => ;
	const async Empty9 => ;
	const async Empty10()[] => ;

	Empty11 => {
		{}
	}
	Empty12 => {
		{{{{{{{{{{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}}}}}}}}}}
	}

	Empty13 => {
		;
	}
	Empty14 => {
		;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
	}
}
