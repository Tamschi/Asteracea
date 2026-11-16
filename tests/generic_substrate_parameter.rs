use asteracea::{components, ContentParentSubstrate, DefaultAccumulationSubstrate, Substrate};

components! {
	#![substrate = S where S: Substrate
		+ DefaultAccumulationSubstrate<'r>
		+ ContentParentSubstrate<'r, S::RenderAccumulatorDefault, &'static str>
	]

	pub Empty => "Hello!";
}
