mod old {
	#![allow(deprecated)]

	asteracea::component! {
		pub Br()() -> Sync

		<br>
	}
}

asteracea::components! {
	pub Br -> web {
		br;
	}

	Br2 -> web {
		br{}
	}

	Br3 -> web {
		br;
		br;
	}

	Br4 -> web {
		br {
			br {
				br; br{}
			}
			br {
				br;
			}
		}
		br;
	}
}
