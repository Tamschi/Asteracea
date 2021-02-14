use rhizome::Node;
use std::iter;
use vec1::{vec1, Vec1};

asteracea::component! {
	Any(
		pub sometimes*?: usize,
		pub some/many+?: usize,
		pub one/any*: usize,
		pub always+: usize,
	)()

	new with {
		let _: &Option<Vec<usize>> = &sometimes;
		let _: &Option<Vec1<usize>> = &many;
		let _: &Vec<usize> = &any;
		let _: &Vec1<usize> = &always;
	}

	[]
}

#[test]
fn check_values() {
	asteracea::component! {
		Outer()()

		<*Any pub any
			*sometimes = {iter::once(1)}
			*sometimes_item = {2}
			*some = {3}
			*many = {iter::once(4)}
			*any = {iter::once(5)}
			*one = {6}
			*always_item = {7}
			*always = {iter::once(8)}
		>
	}

	let any = Outer::new(
		&Node::new_for::<()>().into_arc(),
		Outer::new_args_builder().build(),
	)
	.unwrap()
	.any;

	assert_eq!(any.sometimes, Some(vec![1, 2]));
	assert_eq!(any.many, Some(vec1![3, 4]));
	assert_eq!(any.any, vec![5, 6]);
	assert_eq!(any.always, vec1![7, 8]);
}

#[test]
fn can_omit_optional() {
	asteracea::component! {
		#[allow(dead_code)]
		Outer()()

		<*Any
			*one = {6}
			*always_item = {7}
		>
	};
}
