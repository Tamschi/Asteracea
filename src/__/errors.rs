use std::marker::PhantomData;

use lignin_schema::YesCancelable;

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
