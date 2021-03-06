use std::marker::PhantomData;

use lignin_schema::{NoBubbles, YesBubbles, YesCancelable};

pub struct CheckNoBubbles<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> CheckNoBubbles<T>
where
	T: NoBubbles,
{
	#[inline(always)]
	pub fn check() {}
}

pub struct CheckYesBubbles<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> CheckYesBubbles<T>
where
	T: YesBubbles,
{
	#[inline(always)]
	pub fn check() {}
}

pub struct CheckYesCancelable<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> CheckYesCancelable<T>
where
	T: YesCancelable,
{
	#[inline(always)]
	pub fn check() {}
}

pub trait ActiveNotValid {
	#[deprecated = "Keyword `active` is not valid for this event; the event is not cancelable."]
	#[inline(always)]
	fn check() {}
}
impl<T> ActiveNotValid for T {}

pub trait BubbleNotValid {
	#[deprecated = "Keyword `bubble` is not valid for this event; the event does not bubble."]
	#[inline(always)]
	fn check() {}
}
impl<T> BubbleNotValid for T {}

pub trait CaptureNotValid {
	#[deprecated = "Keyword `capture` is not valid for this event; the event does not bubble."]
	#[inline(always)]
	fn check() {}
}
impl<T> CaptureNotValid for T {}

pub trait PhaseExpected {
	#[deprecated = "Exected one of keywords `bubble` or `capture`; this event bubbles."]
	#[inline(always)]
	fn check() {}
}
impl<T> PhaseExpected for T {}
