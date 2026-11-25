mod _sub;

asteracea::components! {
	#![substrate = _sub::Sub]

	pub RenderFor => [for] _ in 0..2 { }

	// pub ForImplicit => {
	// 	for i in [1, 2, 3, 4, 5i32] {
	// 		"{}"(i);
	// 	}
	// }

	// pub ForImplicitSelector => {
	// 	for i: u8 in 1..=5 {
	// 		"{}"(i);
	// 	}
	// }

	// pub ForImplicitItemType => {
	// 	for i keyed i => u8 in 1..=5 {
	// 		"{}"(i);
	// 	}
	// }

	// pub ForKeyTypeOnly => {
	// 	for i => u8 in &[1, 2, 3, 4, 5] {
	// 		"{}"(i);
	// 	}
	// }

	// pub ForExplicit => {
	// 	for i: u8 keyed i => u8 in [1, 2, 3, 4, 5] {
	// 		"{}"(i);
	// 	}
	// }

	// pub ForUntyped => {
	// 	for i keyed i in [1, 2, 3, 4, 5] {
	// 		"{}"(i);
	// 	}
	// }

	// pub Split => {
	// 	for c in "This is a test.".split(' ') {
	// 		li {
	// 			"{:?}"(c);
	// 		}
	// 		"\n";
	// 	}
	// }

	// pub Child => {
	// 	for _ in 0..5 {
	// 		ForImplicit;
	// 	}
	// }
}
