use asteracea::__::{infer_builder, AnonymousContentParentParameters};

#[test]
fn my_test() {
	let _: AnonymousContentParentParameters = {
		let phantom = [];
		if false {
			<[_; 0] as IntoIterator>::into_iter(phantom).next().unwrap()
		} else {
			infer_builder(
				phantom,
				|builder| -> Result<_, asteracea::error::Escalation> { Ok(builder.build()) },
			)
			.unwrap()
		}
	};
}
