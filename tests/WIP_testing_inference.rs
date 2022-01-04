use asteracea::__::{infer_builder, infer_built, AnonymousContentParentParameters};

#[test]
fn my_test() {
	let _: AnonymousContentParentParameters = infer_built(
		infer_builder(|builder| -> Result<_, asteracea::error::Escalation> { Ok(builder.build()) })
			.unwrap(),
	);
}
