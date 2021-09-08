use std::marker::PhantomData;

use lignin_schema::{NoBubbles, YesBubbles, YesCancelable};

mod private {
	pub enum Invalid {}
}
use private::Invalid;

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
	fn check(_: Invalid) {}
}
impl<T> ActiveNotValid for T {}

pub trait BubbleNotValid {
	#[deprecated = "Keyword `bubble` is not valid for this event; the event does not bubble."]
	#[inline(always)]
	fn check(_: Invalid) {}
}
impl<T> BubbleNotValid for T {}

pub trait CaptureNotValid {
	#[deprecated = "Keyword `capture` is not valid for this event; the event does not bubble."]
	#[inline(always)]
	fn check(_: Invalid) {}
}
impl<T> CaptureNotValid for T {}

pub trait PhaseExpected {
	#[deprecated = "Expected one of keywords `bubble` or `capture`; this event bubbles."]
	#[inline(always)]
	fn check(_: Invalid) {}
}
impl<T> PhaseExpected for T {}
