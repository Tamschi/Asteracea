use asteracea::components;

mod _sub;

components! {
	#![substrate = _sub::Sub]

	Complicated(
		a: i32,
		mut b: i32,
		c as d: i32,
		e: i32 = 1,
		f as mut g: i32 = 2,
		self.h: u32,
	) => ;
}