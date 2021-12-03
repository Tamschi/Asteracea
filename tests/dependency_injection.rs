#![allow(dead_code)]

trait A {}
trait B {}
struct C;

asteracea::component! {
	InjectionUser(
		ref test: dyn A,
		ref test2: &dyn B,
		priv ref test3: C,
	)()

	[]
}
