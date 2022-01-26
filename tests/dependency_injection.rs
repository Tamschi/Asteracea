#![allow(dead_code)]

trait A {}
trait B {}
struct C;

asteracea::component! {
	InjectionUser(
		dyn test: dyn A,
		// dyn ref test2: &dyn B,
		// priv dyn ref test3: C,
	)()

	[]
}
