mod _sub;

use asteracea::Substrate;

asteracea::components! {
	#![substrate = _sub::Sub]

	pub Conditional1[
		present: bool,
	] => [if] present {
		"I am here.";
	}

	pub Conditional2[
		present: bool,
	] => {
		[if] present {
			"I am here";
			span{ " and "; }
		}
		"I like this place.";
	}

	pub Conditional3[
		//TODO: Allow `impl …` types here, efficiently via generic args.
		//TODO: Allow 'i and 'r to be used by the user.
		content?: Box<dyn for<'b> FnOnce() -> <_sub::Sub as Substrate>::RenderTDefault<'static>>,
	] => {
		"[";
		[if] let Some(content) = content {
			[{ content() }]
		}
		"]";
	}
}
