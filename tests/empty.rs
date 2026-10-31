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
	Empty6{} => ;
	Empty7()[] => ;
	Empty8(){} => ;
	Empty9[]{} => ;
	Empty10()[]{} => ;
	async Empty11() => ;
	async Empty12[] => ;
	async Empty13{} => ;
	async Empty14()[] => ;
	async Empty15(){} => ;
	async Empty16[]{} => ;
	async Empty17()[]{} => ;

	Empty18 => {
		{}
	}
	Empty19 => {
		{{{{{{{{{{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}}}}}}}}}}
	}

	Empty20 => {
		;
	}
	Empty21 => {
		;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
	}
}
