//! Bound for use with substrate-generic [`components!`](`crate::components!`) invocations.

use crate::substrate::{ContentParentSubstrate, DefaultAccumulationSubstrate};

pub trait SimpleContentSubstrate<'r, T>:
	DefaultAccumulationSubstrate<'r> + ContentParentSubstrate<'r, Self::RenderAccumulatorDefault, T>
{
}

impl<'r, S, T> SimpleContentSubstrate<'r, T> for S where
	S: DefaultAccumulationSubstrate<'r>
		+ ContentParentSubstrate<'r, Self::RenderAccumulatorDefault, T>
{
}
