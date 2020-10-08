pub trait YankAny<T> {
	fn yank_any(&mut self, predicate: impl FnMut(&T) -> bool) -> Option<T>;
}

impl<T> YankAny<T> for Vec<T> {
	fn yank_any(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<T> {
		self.iter()
			.enumerate()
			.find(|(_, x)| predicate(x))
			.map(|(i, _)| i)
			.map(|i| self.swap_remove(i))
	}
}
