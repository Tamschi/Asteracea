//! `let`-bindings in component bodies.
//!
//! In order to correctly scope a `let`-bindings while other statements yield items, any list of (child) elements of the form
//!
//! ```rust,ignore
//! <a> <b> <c>
//! let foo = bar;
//! <d> <e> // Both `d` and `e` can see `foo` and either can move it.
//! ```
//!
//! must be transformed into an expression of the form:
//!
//! ```rust,ignore
//! ArrayGlue::chain(
//!     [a, b, c,],
//!     {
//!         let foo = bar;
//!         [d, e,]
//!     },
//! )
//! ```
//!
//! (This appears to be inlined into emplacements reasonably well.)

use core::mem::MaybeUninit;

/// Move-concatenates two fixed-length arrays.
pub trait ArrayGlue<T, const N: usize> {
	/// Where `Self` is `[T; M]`, this is `[T; M + N]`.
	type Chained;
	/// Move-concatenates two fixed-length arrays.
	fn chain(self, other: [T; N]) -> Self::Chained;
}

macro_rules! array_impls {
	($(
		($($M:literal),*$(,)?) x ($N:literal)
	),*$(,)?) => {$(
		$(
			impl<T> ArrayGlue<T, $N> for [T; $M] {
				type Chained = [T; $M + $N];
				fn chain(self, other: [T; $N]) -> Self::Chained {
					let mut chained = MaybeUninit::<Self::Chained>::uninit();
					unsafe {
						chained.as_mut_ptr().cast::<[T; $M]>().write(self);
						chained.as_mut_ptr().cast::<T>().offset($M).cast::<[T; $N]>().write(other);
						chained.assume_init()
					}
				}
			}
		)*
	)*};

	($(
		$Ms:tt x ($N_1:literal, $($N:literal),*$(,)?)
	),*$(,)?) => {$(
		array_impls!($Ms x ($N_1));
		$(array_impls!($Ms x ($N));)*
	)*}
}

array_impls!(
	(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32)
	x
	(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32),
);
